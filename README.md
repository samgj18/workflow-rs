**Warp is now available for Linux, archiving in favour of that**

# Workflow

Workflow-rs is a simple `workflow` engine written in Rust. Strictly speaking, it is merely
a parser from a YAML to a Rust structure. The YAML structure choosen is a simple DSL that
allows to define a workflow as a set of tasks and their dependencies. 

Heavily inspired by the great work from [warp](https://docs.warp.dev/getting-started/readme).
Specifically, their [workflow feature](https://docs.warp.dev/features/entry/yaml-workflows).
The idea being to port this to any terminal not only `warp` users, [like myself](https://sw.kovidgoyal.net/kitty/).

## DSL
| Key | Description | Required |
| --- | --- | --- |
| name | The name of the workflow | Yes |
| command | The command to be executed | Yes |
| tags | A list of tags to be associated with the workflow | No |
| description | A description of the workflow | No |
| arguments | A list of arguments to be passed to the command | No |
| source_url | The URL of the source code of the workflow | No |
| author | The author of the workflow | No | | |
| author_url | The URL of the author of the workflow | No |

### Arguments
| Key | Description | Required |
| --- | --- | --- |
| name | The name of the argument | Yes |
| description | A description of the argument | No |
| default_value | The default value of the argument | No |
| values | A list of possible values for the argument | No |

## Example

```yaml
---
name: Echo a message with a list of arguments
command: |-
  echo "This is a cool echo to try out: {{sshKeyPath}} and User: {{userName}} <{{userEmail}}>"
tags:
  - ssh
  - echo
description: Clones a git repository given a specific SSH Key Path and configures it to use the desired Name and Email
arguments:
  - name: sshKeyPath
    description: The path of the SSH Key to be used
    default_value: ~/.ssh/id_rsa
    values:
      - ~/.ssh/id_rsa
      - ~/.zshrc.zwc/
  - name: repositoryUrl
    description: The SSH URL of the git repository
  - name: targetFolder
    description: The name of the folder in which the repository should be cloned into
  - name: userName
    description: The Name of the User to be configured for the git repository
    default_value: Jhon Doe
  - name: userEmail
    description: The Email of the User to be configured for the git repository
    default_value: johndoe@example.com
source_url: "https://github.com/samgj18/echo"
author: samuel
author_url: "https://github.com/samgj18"
```

## Installation

```bash
cargo install workflow
```

## Roadmap

See the [open issues]([https://github.com/samgj18/workflow-rs/issues](https://github.com/samgj18/workflow-rs/issues?q=is%3Aopen+is%3Aissue+author%3Asamgj18+label%3Aenhancement)) for a list of proposed features (and known issues).

## Usage

By default, the work directory used is `$HOME/.workflows/`. This can be changed by setting the `WORKFLOW_DIR` environment variable
or by adding a `workflow_dir` key to the `workflow.txt` file inside `$HOME/.config/workflows/`.

```txt
# Workflow configuration file => This is a comment and will be ignored
workflow_dir = your/workflow/dir
```

The hyerarchy of the workflow directory is as follows:

1. `WORKFLOW_DIR`
2. `$HOME/.config/workflows/`
3. `$HOME/.workflows/`

```bash
workflow --help 
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
