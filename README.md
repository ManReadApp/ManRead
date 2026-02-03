# ManRead
My goal with this project was to create a manga reader which has capabilities to translate mangas and will download the optimal version of the manga. Its goal is to maximize the quality at cost of setup convenience & storage.

## About the project
This project is at its early stages and is still in development. Features like migration & auto updater are still not implemented and wont be until we reach a stable version or bigger user base. Il implement this at probably 100 stars, but it doesnt really make sense as major changes will still happen. Stuff like the scrapers are for from stable and in case i swap out the db again it´l be a pain to auto migrate. A migration bash script is just way easier to work with & i dont need to worry about other operating systems(aka windows)

## Tech
### Languages
- Rust
- TypeScript
- my own scraping syntax ;D

### Frameworks
- [NuxtJS](https://nuxtjs.org)/[Vue](https://vuejs.org)
- [ActixWeb](https://actix.rs)
- [OpenApi](https://swagger.io)
- [TailwindCSS](https://tailwindcss.com)
- [Tauri](https://v2.tauri.app)

### Databases
- [SurrealDB](https://surrealdb.com)

## Contributing
Contributions are welcome! Especially if you have experience with Vue/NuxtJS/Webdesign. I have no clue about design and would love if someone could actually help fix the inconsistencies in the design. Backend help is also appreciated, but the state is way better than the frontend ;D. I might have overcomplicated quite a lot of things, while other areas lack complexity where it would be quite useful. Please follow the guidelines below:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and commit them.
4. Push your changes to your fork.
5. run `cargo fmt --all` & `cargo clippy`
6. Create a pull request.

## Goals for 1.0
- new ui
- user friendly search
- backend full capabilities(no more todos)
- no more unwrap
- general cleanup
- auto update
- auto migration
- docker & kubernetes
- fix sql injection bugs ;D
- stable interface for plugins
- set rust version in Cargo.toml(bc external "Rust")
- shrink plugin size(15mb is quite large)

## SETUP
1. start backend `cargo r --release` or run binary
2. place some example covers in `data/cover_templates` which will be used if no cover is found on user creation
3. start frontend `cd frontend && pnpm install && pnpm run start`
3. start frontend `cd frontend && pnpm install && pnpm tauri dev`
4. create a user with the activation code `000000`

## Usage
- Use a modern browser
  - needs to support WASM which all modern browsers do
- Navigate to `http://127.0.0.1:3000`

## Search syntax
- Exmaple: `"This is a title"` <-- ordered
- Exmaple: `This is a title` <-- unordered
- Example: `(title:"This is a title" | title:"This is another title") & author:"This is an author"` <-- advanced
- | is or
- & is and
- () is a group
- !: is not
- !() is a not group
- &() joins items in a group with an and
- |() joins items in a group with an or
