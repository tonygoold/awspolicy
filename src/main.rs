use awspolicy::iam::Action;
use awspolicy::policy::{sample_policy, Policy};
use awspolicy::aws::ARN;

use clap::Parser;
use json;

#[derive(Parser, Debug)]
#[clap(about, version)]
struct Args {
    #[clap(long)]
    policy: Option<String>,

    #[clap(long)]
    principal: Option<String>,

    #[clap(long)]
    action: Option<String>,

    #[clap(long)]
    resource: Option<String>,
}

fn load_policy(path: Option<String>) -> json::Result<Policy> {
    let path = if let Some(p) = path { p } else { return sample_policy(); };
    let data = std::fs::read_to_string(&path).map_err(|_| json::Error::wrong_type("unable to read policy file"))?;
    Policy::try_from(data.as_str())
}

fn check_action(policy: &Policy, action: &str, resource: &str) {
    let action = match Action::try_from(action) {
        Ok(action) => action,
        Err(err) => {
            println!("Invalid action: {:?}", err);
            return;
        }
    };
    let resource = match ARN::try_from(resource) {
        Ok(resource) => resource,
        Err(err) => {
            println!("Invalid resource: {:?}", err);
            return;
        }
    };
    let result = policy.check(&action, &resource);
    println!("Checked {:?} on {:?}: {:?}", &action, &resource, result);
}

fn main() {
    let args = Args::parse();
    let policy = match load_policy(args.policy) {
        Ok(policy) => policy,
        Err(err) => {
            println!("Policy parse error: {:?}", err);
            return;
        }
    };
    match (args.action, args.resource) {
        (Some(action), Some(resource)) => check_action(&policy, &action, &resource),
        (Some(_), None) => println!("You must specify a resource for the action"),
        (None, Some(_)) => println!("You must specify an action for the resource"),
        _ => println!("Policy parsed"),
    }
    // match sample_policy() {
    //     Ok(policy) => {
    //         println!("{:?}\n", &policy);
    //         let action = Action::new("route53", "GetChange");
    //         let resource = ARN::new("route53", "", "", "change/Foo");
    //         let result = policy.check(&action, &resource);
    //         println!("Checked {:?} on {:?}: {:?}", &action, &resource, result);
    //     }
    //     Err(err) => println!("Policy parse error: {:?}", err),
    // }
}
