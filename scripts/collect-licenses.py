#!/usr/bin/env python3
#
# Regenerates THIRD-PARTY-NOTICES.md from what is actually in the dependency
# graph, rather than from a list somebody remembers to update.
#
# Marswind is MIT, and almost everything under it is MIT or Apache-2.0 - which
# is to say almost everything under it asks for its copyright notice to travel
# with the binary. The .app ships this file as a resource for that reason, and
# an inventory typed out by hand is one that stops being true at the first
# `cargo update`.
#
# Sources of truth, in order of how much they can be trusted:
#
#   - `cargo metadata` for the Rust graph. It reads the lockfiles, so it lists
#     the exact versions that get compiled, on every platform rather than only
#     this one - a Windows-only crate is still a crate this project depends on.
#   - `npm ls` for which JavaScript packages are installed and where, and then
#     each package's own `package.json` for what it is licensed under. The
#     fields `npm ls --long` echoes back are not filled in consistently enough
#     to put in a file that ships.
#   - a hand-written table for the C and C++ libraries. whisper.cpp, llama.cpp
#     and ggml are compiled into the binaries out of the `-sys` crates that
#     vendor them, so Cargo knows nothing about them beyond the crate that
#     carries them.
#
# The output is reproducible: the same lockfiles produce the same file, so a
# diff after running this means the graph moved and the notices inside the
# shipped app were about to be wrong.
#
# Usage: npm run licenses

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# whisper.cpp, llama.cpp and ggml arrive as vendored source trees inside
# whisper-rs-sys and llama-cpp-sys-2 and are compiled straight into the two
# binaries. What is written down here is what cannot be derived: who holds the
# copyright, and where the project lives.
#
# Their own version numbers mostly cannot be derived either. whisper.cpp states
# one in its CMakeLists and is read from there; llama.cpp computes its from a
# git commit count that a published crate no longer has, and ggml never states
# one at all. Those fall back to the version of the `-sys` crate carrying them,
# which is the number that actually pins what gets compiled.
NATIVE = [
    {
        "name": "whisper.cpp",
        "license": "MIT",
        "copyright": "Copyright (c) 2023-2024 The ggml authors",
        "url": "https://github.com/ggml-org/whisper.cpp",
        "used_by": "Marswind",
        "note": "Speech recognition, and the Silero VAD implementation with it.",
        # The crate that vendors it, and where inside that crate to look for a
        # version. A `None` path means the library does not state one.
        "carrier": "whisper-rs-sys",
        "probe": "whisper.cpp/CMakeLists.txt",
    },
    {
        "name": "llama.cpp",
        "license": "MIT",
        "copyright": "Copyright (c) 2023-2024 The ggml authors",
        "url": "https://github.com/ggml-org/llama.cpp",
        "used_by": "marswind-translator",
        "note": "Runs the translation model. Built with the server, tools and "
        "examples off, so the libraries vendored for those - cpp-httplib, "
        "miniaudio, nlohmann/json, stb - are not compiled or linked.",
        "carrier": "llama-cpp-sys-2",
        "probe": None,
    },
    {
        "name": "ggml",
        "license": "MIT",
        "copyright": "Copyright (c) 2023-2024 The ggml authors",
        "url": "https://github.com/ggml-org/ggml",
        "used_by": "both",
        "note": "The tensor library under both of the above, including the "
        "Metal backend. Each ships its own copy, which is why the two "
        "cannot share a process - see docs/ARCHITECTURE.md.",
        "carrier": None,
        "probe": None,
    },
]

# Licenses that need nothing from us beyond being named. Anything outside this
# set is worth a human looking at before a release, so it gets reported.
EXPECTED = {
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "Unlicense",
    "0BSD",
    "MIT-0",
    "CC0-1.0",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "MPL-2.0",
    "CDLA-Permissive-2.0",
    # The LLVM exception only removes an obligation Apache-2.0 imposes on
    # people redistributing derived works, so it is Apache-2.0 or better.
    "Apache-2.0 WITH LLVM-exception",
    "BlueOak-1.0.0",
    "Python-2.0",
    "WTFPL",
}


def run(args, **kwargs):
    return subprocess.run(
        args, cwd=ROOT, capture_output=True, text=True, check=True, **kwargs
    ).stdout


def cargo_packages(manifest):
    """Every crate in one lockfile's graph, the local crates excluded."""
    meta = json.loads(
        run(
            [
                "cargo",
                "metadata",
                "--format-version",
                "1",
                "--manifest-path",
                manifest,
            ]
        )
    )
    local = {member.split()[0] for member in meta["workspace_members"]}
    packages = {}
    for package in meta["packages"]:
        if package["name"] in local or package.get("source") is None:
            continue
        packages[(package["name"], package["version"])] = {
            "license": package.get("license") or license_from_file(package),
            "url": package.get("repository") or "",
        }
    return packages


def license_from_file(package):
    """A crate with `license-file` instead of `license` says so here."""
    return f"see {package['license_file']}" if package.get("license_file") else "?"


def npm_packages():
    """Every installed npm package, dev tooling included.

    `npm ls --all` fails with a non-zero status on any peer-dependency
    complaint while still printing a perfectly good tree, so its output is read
    rather than its exit code.

    What the tree is trusted for is the set of packages and where each one
    lives, and nothing else. The `license` and `repository` fields `--long`
    echoes back are populated inconsistently - the same lockfile installed
    twice produced notices with forty-four repository links in one copy and
    none in the other - so both are read from the package's own `package.json`
    on disk, which is the file npm is summarising in the first place.
    """
    result = subprocess.run(
        ["npm", "ls", "--all", "--json", "--long"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    try:
        tree = json.loads(result.stdout)
    except json.JSONDecodeError:
        sys.exit("npm ls produced no tree - run `npm install` first")

    packages = {}

    def walk(node):
        for name, child in (node.get("dependencies") or {}).items():
            version = child.get("version")
            if not version:
                continue
            packages[(name, version)] = manifest(child.get("path"), child)
            walk(child)

    walk(tree)
    return packages


def manifest(path, node):
    """License and repository for one npm package, from its own package.json."""
    data = {}
    if path:
        file = Path(path) / "package.json"
        if file.is_file():
            try:
                data = json.loads(file.read_text(errors="ignore"))
            except json.JSONDecodeError:
                pass

    license_ = data.get("license") or node.get("license")
    if isinstance(license_, dict):  # the old {"type": …} spelling
        license_ = license_.get("type")
    if not license_ and isinstance(data.get("licenses"), list):
        # Older still: a list of them, which means a choice.
        license_ = " OR ".join(
            entry.get("type", "?")
            for entry in data["licenses"]
            if isinstance(entry, dict)
        )

    repository = data.get("repository") or node.get("repository")
    if isinstance(repository, dict):
        repository = repository.get("url", "")

    return {
        "license": license_ or "?",
        "url": normalize_repo(repository or ""),
    }


def normalize_repo(url):
    """`git+ssh://git@github.com/a/b.git` is not a link anyone can follow."""
    url = re.sub(r"^git\+", "", url)
    url = re.sub(r"^git://", "https://", url)
    url = re.sub(r"^ssh://git@", "https://", url)
    url = re.sub(r"^git@([^:]+):", r"https://\1/", url)
    return re.sub(r"\.git$", "", url)


def native_version(entry, crates):
    """What to print in the Version column for a vendored C library.

    Its own number when the library states one, and the version of the crate
    that vendors it otherwise - that is the number a lockfile pins, so it is
    the one that says which source tree was compiled.
    """
    carrier = entry["carrier"]
    carrier_version = next(
        (version for (name, version) in crates if name == carrier), None
    )

    if entry["probe"] and carrier_version:
        registry = Path.home() / ".cargo/registry/src"
        for index in registry.glob("*"):
            cmake = index / f"{carrier}-{carrier_version}" / entry["probe"]
            if cmake.is_file():
                match = re.search(
                    r"project\(\s*\"?[\w.+-]+\"?\s+VERSION\s+([\d.]+)",
                    cmake.read_text(errors="ignore"),
                )
                if match:
                    return match.group(1)

    if carrier_version:
        return f"vendored by `{carrier}` {carrier_version}"
    return "vendored by both of the above"


def needs_review(packages):
    """Packages whose license offers no permissive way to take them.

    Nearly every crate here is `MIT OR Apache-2.0` or similar, and a handful
    offer a copyleft alternative beside a permissive one - `self_cell` is
    `Apache-2.0 OR GPL-2.0-only`. Splitting on the operators and reporting every
    term would flag those as GPL, which is the opposite of what an expression
    with an OR in it means. So each alternative is checked whole: if one of them
    is entirely permissive, the package is taken under that one and there is
    nothing to report.
    """
    flagged = {}
    for (name, version), entry in packages.items():
        expression = entry["license"]
        alternatives = re.split(r"\s+OR\s+|/", expression.replace("(", " ").replace(")", " "))
        if any(
            all(
                term.strip() in EXPECTED
                for term in re.split(r"\s+AND\s+", alternative)
                if term.strip()
            )
            and alternative.strip()
            for alternative in alternatives
        ):
            continue
        flagged[f"{name} {version}"] = expression
    return flagged


def table(packages, url_column=True):
    lines = ["| Package | Version | License |" + (" Source |" if url_column else "")]
    lines.append("|---|---|---|" + ("---|" if url_column else ""))
    for (name, version), entry in sorted(packages.items(), key=lambda kv: kv[0][0]):
        row = f"| `{name}` | {version} | {entry['license']} |"
        if url_column:
            url = entry["url"]
            row += f" [link]({url}) |" if url.startswith("http") else " |"
        lines.append(row)
    return "\n".join(lines)


def main():
    app = cargo_packages("src-tauri/Cargo.toml")
    worker = cargo_packages("translator/Cargo.toml")
    crates = {**worker, **app}
    npm = npm_packages()

    for entry in NATIVE:
        entry["version"] = native_version(entry, crates)

    flagged = {**needs_review(crates), **needs_review(npm)}

    out = ROOT / "THIRD-PARTY-NOTICES.md"
    out.write_text(render(crates, npm, flagged))

    print(f"wrote {out.relative_to(ROOT)}")
    print(f"    {len(crates)} Rust crates, {len(npm)} npm packages")
    if flagged:
        print("    no permissive option, read these before releasing:")
        for package, expression in sorted(flagged.items()):
            print(f"        {package}: {expression}")
    else:
        print("    every package can be taken under a permissive license")


def render(crates, npm, flagged):
    native_rows = "\n".join(
        f"| [{e['name']}]({e['url']}) | {e['version']} | {e['license']} |"
        f" {e['used_by']} | {e['note']} |"
        for e in NATIVE
    )
    native_copyrights = "\n".join(
        f"- {e['name']} - {e['copyright']}" for e in NATIVE
    )
    warning = (
        ""
        if not flagged
        else "\n> **Packages with no permissive option**, which need a decision rather"
        " than a\n> notice:\n>\n"
        + "\n".join(f"> - `{package}` - {terms}" for package, terms in sorted(flagged.items()))
        + "\n"
    )

    return f"""# Third-party notices

Marswind is MIT licensed (see [LICENSE](LICENSE)). It is built out of other
people's work, most of it under licenses that ask for their copyright notice to
travel with the binary. This file is that notice, and it is shipped inside
`Marswind.app` alongside the license itself.

**It is generated.** Run `npm run licenses` after changing a dependency; do not
edit it by hand. The Rust half comes from `cargo metadata`, so it lists what the
lockfiles actually resolve to, on every platform rather than only the one you
are building on.

Models are **not** covered here. They are downloaded on your request, from
Hugging Face, and carry their own licenses - see
[docs/MODELS.md](docs/MODELS.md).
{warning}
## Compiled into the binaries

C and C++ libraries vendored inside their `-sys` crates and compiled straight
into `Marswind` and `marswind-translator`. Cargo does not list them separately,
so they are named here.

| Library | Version | License | Used by | What for |
|---|---|---|---|---|
{native_rows}

Copyright holders, as stated in each project's own LICENSE file:

{native_copyrights}

## Rust crates

{len(crates)} crates across both binaries, from the `src-tauri/Cargo.lock` and
`translator/Cargo.lock` graphs. Where a crate offers a choice of licenses
(`MIT OR Apache-2.0` and friends), Marswind takes it under whichever is
compatible with MIT; the choice is not narrowed here because the text is the
crate's to state.

{table(crates)}

## npm packages

{len(npm)} packages. Most of these are build tooling and no part of them reaches
the shipped app - the exceptions are the Svelte runtime and `@tauri-apps/api`,
which are compiled into the JavaScript bundle. They are listed together because
"which bundler output contains which module" is not a question a dependency tree
can answer honestly.

{table(npm)}
"""


if __name__ == "__main__":
    main()
