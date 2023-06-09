# vectorizer
A personal WIP tool to create text embeddings for project files and upload them to Qdrant DB for use with the ChatGPT retrieval plugin

The reason for this tool is to be able to upload project files without having to hit OpenAI api's every time I wanted to upload something.

While the tool works, there is some setup required and probably some bugs yet to be discovered. I would not recommend using it for anything serious, but I can't tell you what to do with your life.


> **Note**
> cargo run works fine, but to build and run standalone, it requires libtorch and for the appropriate env vars to be set, which can be seen in the .env file.

Running `cargo run` will automatically download the appropriate libtorch version, and you could then simply copy it to a permanent location and set the env vars to that directory.
libtorch can be found in the following location after that:
`target/**/build/torch-sys-**/out/libtorch`

Uses the All-MiniLm-(L12/L6)-V2 model for generating the text embeddings. Currently L12, but I could make it selectable as to which one to use if needed. It also downloads and caches the model, but this causes a small extra overhead each run.  

This can be mitigated by persisting the model and specifying it's location. Currently, this must be hard coded in vectorize.rs, but I could also expose this as a setting.  

I have not done so because the tool was just for me and I am fine with how it is. If someone else requests this change, I can definitely make it happen.   

---

### Main Configuration
```toml
# File Name:   $HOME/.config/vectorizer/settings.toml
# File Format: Toml

[indexer]
log_level = "warn"
project_file = false # Create a project settings override file automatically if you run vectorizer on a directory
extensions   = []    # List of file extensions to index
directories  = []    # List of directories to include within the project root
ignored      = []    # List of directories to ignore within the project root

[database]
url          = ""    # URL to the database (required)
collection   = ""    # Name of the collection to create/use
max_tokens   = 256   # Maximum tokens per fragment when splitting documents
metadata     = ""    # Additional Metadata to add, in json format - ex: '{"language":"rust", "content", "source code"}'
```

### Per Project Configuration

To have specific settings per project, create a .vectorizer file in the project root
Then run `vectorizer -p /path/to/project/root --upload`

```toml
# File Name:   .vectorizer
# File Format: Toml

# Only include the options you wish to override from the main configuration
# Example:

[indexer]
extensions   = ["rs"]  

[database]
metadata     = '{"language":"rust", "content", "source code"}' 

```

### Neovim

If you want to async auto upsert the current buffer when you save them:

```lua
-- Requires plenary.nvim package
local Job = require('plenary.job')
local osEnv = {}

for line in io.popen("set"):lines() do
  local envName = line:match("^[^=]+")
  osEnv[envName] = os.getenv(envName)
end

local function RunJob()
  local cwd = vim.fn.getcwd()
  local file = vim.fn.expand('%:p')

  Job:new({
    command = 'vectorizer',
    args = {'-p', file, '--upload' },
    cwd = cwd,
    env = osEnv,
    on_exit = function(j, return_val)
      print(vim.inspect(return_val))
      print(vim.inspect(j:result()))
    end,
    on_stderr = function(_, output) print(output) end,
  }):start()
end

vim.api.nvim_create_autocmd("BufWritePost", {
  callback = function()
    RunJob()
  end,
})
```

---
![alt text](https://i.imgur.com/cg5ow2M.png "instance.id")

