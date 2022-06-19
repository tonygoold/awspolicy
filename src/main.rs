use awspolicy::iam::{Action, Principal};
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
    principal_aws: Option<String>,

    #[clap(long)]
    principal_federated: Option<String>,

    #[clap(long)]
    principal_service: Option<String>,

    #[clap(long)]
    principal_canonical_user: Option<String>,

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
    let result = policy.check_action(&action, &resource);
    println!("Checked {:?} on {:?}: {:?}", &action, &resource, result);
}

fn check_policy(policy: &Policy, principal: &Principal, action: &str, resource: &str) {
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
    let result = policy.check(principal, &action, &resource);
    println!("Checked {:?} by {:?} on {:?}: {:?}", &action, &principal, &resource, result);
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
    let principal = match (args.principal_aws, args.principal_service, args.principal_federated, args.principal_canonical_user) {
        (Some(aws), None, None, None) => if let Ok(arn) = ARN::try_from(aws.as_str()) {
            Some(Principal::AWS(arn))
        } else {
            println!("Invalid AWS principal");
            return;
        }
        (None, Some(service), None, None) => Some(Principal::Service(service)),
        (None, None, Some(federated), None) => Some(Principal::Federated(federated)),
        (None, None, None, Some(canonical)) => Some(Principal::CanonicalUser(canonical)),
        (None, None, None, None) => None,
        _ => {
            println!("You can only specify one type of principal");
            return;
        }
    };
    match (args.action, args.resource) {
        (Some(action), Some(resource)) => if let Some(principal) = principal {
            check_policy(&policy, &principal, &action, &resource)
        } else {
            check_action(&policy, &action, &resource)
        }
        (Some(_), None) => println!("You must specify a resource for the action"),
        (None, Some(_)) => println!("You must specify an action for the resource"),
        _ => println!("Policy parsed"),
    }
}
