use std::collections::HashMap;

use clap::Parser;
use inquire::Text;
use workflow::domain::prelude::*;

pub trait WorkflowExecutor {
    fn get(&self, name: &str) -> Result<Workflow, Error>;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The name of the workflow")]
    name: String,
}

pub enum FileExtension {
    Yaml,
    Yml,
    None,
}
impl<'a> From<&'a str> for FileExtension {
    fn from(value: &'a str) -> Self {
        if value.contains(".yml") {
            FileExtension::Yml
        } else if value.contains(".yaml") {
            FileExtension::Yaml
        } else {
            FileExtension::None
        }
    }
}

const WORKDIR: &str = "specs";
impl WorkflowExecutor for Args {
    fn get(&self, name: &str) -> Result<Workflow, Error> {
        let mut values = vec![];

        match FileExtension::from(name) {
            FileExtension::Yaml => values.push(name.to_string()),
            FileExtension::Yml => values.push(name.to_string()),
            FileExtension::None => values.push(format!("{}.yaml", name)),
        }

        values
            .iter()
            .find_map(|value| {
                let workflow = std::fs::read_to_string(format!("{}/{}", WORKDIR, value))
                    .map_err(|e| Error::ReadError(Some(e.into())));

                let workflow = serde_yaml::from_str::<Workflow>(&workflow.unwrap())
                    .map_err(|e| Error::ParseError(Some(e.into())));

                match workflow {
                    Ok(workflow) => Some(Ok(workflow)),
                    Err(_) => None,
                }
            })
            .unwrap_or_else(|| Err(Error::InvalidName(None)))
    }
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    // Read the workflow from the file system
    let workflow = args.get(&args.name)?;
    println!("{:?}", workflow.command().inner());

    let precedence = workflow.arguments().iter().try_fold(
        HashMap::new(),
        |mut acc, argument| -> Result<HashMap<String, String>, Error> {
            let value = inquire::Text::new(argument.name().inner())
                .prompt()
                .map_err(|e| Error::ReadError(Some(e.into())))?;

            if !value.is_empty() {
                acc.insert(argument.name().inner().to_string(), value);
            }
            Ok(acc)
        },
    )?;

    let args = workflow.parsed(
        &precedence
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect(),
    );

    let command = workflow.command().replace(&args)?;

    println!("{}", command);
    // let template = handlebars::Handlebars::new();
    // let arguments = workflow.parsed(&HashMap::new());

    // let rendered = template.render_template(workflow.command().inner(), &workflow.arguments());
    // println!("Rendered: {}", rendered.unwrap());
    // println!("Workflow: {:?}", workflow);

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
