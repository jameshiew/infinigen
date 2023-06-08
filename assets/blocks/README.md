# blocks

Provided textures are derived from images generated with Midjourney.

- each block must have a `.ron` file in this directory (the name of the file is not important)
- it must include a `name` field
- it can include a `textures` field specifying the texture for each face, otherwise `textures/{name}.png` will be used for everything
