# Web Version

## Build Environment

### Installing `wasm-pack` On Windows
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

## Building For The Web
Once your build environment is set up, run `wasm-pack build` from the project root.

## Test HTTP Server
For testing in the browser, you need a http server. I have used the following for testing:

```powershell
# Run from project root
cargo install basic-http-server
basic-http-server
```
