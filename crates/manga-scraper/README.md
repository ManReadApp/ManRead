## File types

### {uri}.filter
This file is used to register a new external service. It is requierd for `.search` & `.metadata`. If .filter is used a icon is required too.
Earch line is a entry. possible line prefixes are `use`, `starts_with`, `regex`.

- `starts_with` checks if a url starts with the given string.
- `regex` checks if a url matches the given regex.
- `use` adds a processor to the list of processors that will be used when scraping.

### {uri}.qoi, .webp .png .jpg .jpeg .gif .ico .afiv, .svg
Image files(required if filter exists)

### {uri}.search

### {uri}.metadata
The selector system is indent based
there is always a strucure like this
`{name?}[{mode}] {query?}`

There are some specifal chars.
- `@` Array
  - `name_array[@text] .name` querySelectorAll
  - `name[text] .name` querySelector
- `=>` HashMap
  - `[text] .name => [@text] .name` means that the data structure will be a hashmap
- `|` OR. there can be multiple queries for a single field
  - `!`means override kind for multiple queries( only works with `|`)
    - `title_field[text] .title | ![strip_text] .sub_title`
    - the @ symbol only works at the first []
- `&` AND (useless?!)
- the query is a selector like document.querySelectorAll
  - `<-`selects the parent node
- possible modes are `text`, `strip_text`, `html`, `regex`
- \n+indents means that the query will be executed adter it. If there is no name & no hashmap it will be flattend

### {id}.header
This is a json file that contains the headers that will be used when scraping.
