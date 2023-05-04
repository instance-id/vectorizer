# vectorizer
A personal WIP tool to create text embeddings for project files and upload them to Qdrant DB for use with the ChatGPT retrieval plugin

The reason for this tool is to be able to upload project files without having to hit OpenAI api's every time I wanted to upload something.

While the tool works, there is some setup required,and bugs to work out. I would not recommend using it yet, but I can't tell you what to do with your life.


> **Note**
> cargo run works fine, but to build and run standalone, it requires libtorch and for the appropriate env vars to be set, which can be seen in the .env file.

Running `cargo run` will automatically download the appropriate libtorch version, and you could then simply copy it to a permanent location and set the env vars to that directory.
libtorch can be found in the following location after that:
`target/**/build/torch-sys-**/out/libtorch`

Uses the AllMiniLmL12V2 model for generating the text embeddings.

---
![alt text](https://i.imgur.com/cg5ow2M.png "instance.id")
