# Overview
The web version of this project is built using `wasm-pack`. The output from `wasm-pack` and the 
[index.html](../index.html) file are all that is needed to host this project. This repo is set up to 
host the web version using Github pages 

# How To Create Web Builds
Run `wasm-pack` from the project root. The output will appear in the `pkg` directory. See the 
[Build Environment Setup](#build-environment-setup) section below.

# Github Pages
The web version of this project is hosted on Github pages using this repo. The `gh_pages` branch is 
used for publishing. The project root is used as the directory to host. The 
[index.html](../index.html) references files in the [github_pages](../github_pages) directory. The 
files in this directory are manually copied in from `wasm-pack` builds.

# Build Environment Setup

## Installing `wasm-pack` On Windows
`wasm-pack` can be installed using `cargo install wasm-pack`. 

When I first tried this it failed with: 

```plaintext
error: failed to run custom build command for `openssl-sys v0.9.80`
```

If this occurs, you can work around the issue by:

1. manually installing SSL
2. setting the environment variable `OPENSSL_DIR`
3. re-running `cargo install wasm-pack`
re-runing the `cargo install`

Below the PowerShell commands I used to accomplish this:

```powershell
# Install vcpkg a package manager which can be used to install SSL on windows
git clone https://github.com/Microsoft/vcpkg.git
cd .\vcpkg
.\bootstrap-vcpkg.bat
./vcpkg.exe install openssl:x64-windows

# Set OPENSSL_DIR environment variable which is used to install the openssl crate
$env:OPENSSL_DIR="C:\repo\vcpkg\installed\x64-windows"

# Install wasm-pack with cargo
cargo install wasm-pack
```

Further information around building the openssl create can be found here 
https://docs.rs/openssl/latest/openssl/.

## Testing Locally
For testing in the browser, you need a http server. I have used the following for testing:

```powershell
# Run from project root
cargo install basic-http-server
basic-http-server
```
