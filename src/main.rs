use clap::Parser;
use workflow::prelude::*;

fn main() -> Result<Unit, Error> {
    let command: Command = Command::parse();
    let workflow = command.prepare()?;
    command.execute(workflow)?;

    Ok(())
}

// fn yaml() -> &'static str {
//     r#"
// ---
// name: Clone git repository with specific SSH Key and User
// command: |-
//   git -c core.sshCommand='ssh -i {{sshKeyPath}} -o IdentitiesOnly=yes' clone {{repositoryUrl}} {{targetFolder}}
//   cd {{targetFolder}}
//   git config core.sshCommand 'ssh -i {{sshKeyPath}}'
//   git config user.name "{{userName}}"
//   git config user.email {{userEmail}}
// tags:
//   - git
//   - ssh
// description: Clones a git repository given a specific SSH Key Path and configures it to use the desired Name and Email
// arguments:
//   - name: sshKeyPath
//     description: The path of the SSH Key to be used
//     default_value: ~/.ssh/id_rsa
//   - name: repositoryUrl
//     description: The SSH URL of the git repository
//     default_value: <repo_url>
//   - name: targetFolder
//     description: The name of the folder in which the repository should be cloned into
//     default_value: <target_folder>
//   - name: userName
//     description: The Name of the User to be configured for the git repository
//     default_value: Jhon Doe
//   - name: userEmail
//     description: The Email of the User to be configured for the git repository
//     default_value: johndoe@example.com
// source_url: "https://github.com/charlieVader/warp-workflows/blob/master/git/clone-with-ssh.yaml"
// author: charlieVader
// author_url: "https://github.com/charlieVader"
// shells: []
// "#
// }
