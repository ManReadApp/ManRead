# ManRead

A mangareader completly written in rust
## Preview
### Login/SignUp/Forgot Password
![Auth GIF](assets/auth.gif)
## Download
```sh
curl -O https://raw.githubusercontent.com/ManReadApp/ManRead/master/install.sh && bash install.sh
```
download `https://huggingface.co/datasets/GriddleDean/mangaupdates/resolve/main/postgres.sql?download=true` and place in `data/external`


## Run Server
```sh
cd api
cargo r --release
```

#### Run Client
```sh
cd app
cargo r --release
```