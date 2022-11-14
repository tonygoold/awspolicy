use awspolicy::aws::ARN;
use awspolicy::iam::{Action, Principal};
use awspolicy::policy::context::Context;
use awspolicy::policy::{CheckResult, Policy};

use anyhow::anyhow;
use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgsError {
    NoActionSpecified,
    NoResourceSpecified,
    MultiplePrincipalsSpecified,
    InvalidPrincipal,
    InvalidAction,
    InvalidResource,
    InvalidContext,
}

enum RunConfig {
    None,
    Identity(Action, ARN, Context),
    Resource(Principal, Action, ARN, Context),
}

impl RunConfig {
    fn check(&self, policy: &Policy) -> anyhow::Result<CheckResult> {
        match self {
            Self::None => Ok(CheckResult::Unspecified),
            Self::Identity(action, resource, context) => policy.check_action(action, resource, context),
            Self::Resource(principal, action, resource, context) => policy.check(principal, action, resource, context),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(about, version)]
struct Args {
    #[clap(long)]
    policy: String,

    #[clap(long)]
    context: Option<String>,

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

impl TryFrom<&Args> for RunConfig {
    type Error = ArgsError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        if args.action.is_none() && args.resource.is_none() && args.principal_aws.is_none() && args.principal_federated.is_none() && args.principal_service.is_none() && args.principal_canonical_user.is_none() {
            return Ok(RunConfig::None);
        }

        let action = args.action.as_ref().ok_or(ArgsError::NoActionSpecified).and_then(
            |action| Action::try_from(action.as_str()).map_err(|_| ArgsError::InvalidAction)
        )?;
        let resource = args.resource.as_ref().ok_or(ArgsError::NoResourceSpecified).and_then(
            |resource| ARN::try_from(resource.as_str()).map_err(|_| ArgsError::InvalidResource)
        )?;
        let context = args.context.as_ref()
            .map(|path| load_context(path.as_str()))
            .unwrap_or_else(|| Ok(Context::new()))
            .map_err(|_| ArgsError::InvalidContext)?;

        match (&args.principal_aws, &args.principal_service, &args.principal_federated, &args.principal_canonical_user) {
            (Some(aws), None, None, None) => if let Ok(arn) = ARN::try_from(aws.as_str()) {
                Ok(RunConfig::Resource(Principal::AWS(arn), action, resource, context))
            } else {
                Err(ArgsError::InvalidPrincipal)
            }
            (None, Some(service), None, None) => Ok(RunConfig::Resource(Principal::Service(service.clone()), action, resource, context)),
            (None, None, Some(federated), None) => Ok(RunConfig::Resource(Principal::Federated(federated.clone()), action, resource, context)),
            (None, None, None, Some(canonical)) => Ok(RunConfig::Resource(Principal::CanonicalUser(canonical.clone()), action, resource, context)),
            (None, None, None, None) => Ok(RunConfig::Identity(action, resource, context)),
            _ => Err(ArgsError::MultiplePrincipalsSpecified),
        }
    }

}

fn load_policy(path: &str) -> anyhow::Result<Policy> {
    let data = std::fs::read_to_string(path).map_err(|_| anyhow!("unable to read policy file"))?;
    Policy::try_from(data.as_str())
}

fn load_context(path: &str) -> anyhow::Result<Context> {
    let data = std::fs::read_to_string(path).map_err(|_| anyhow!("unable to read context file"))?;
    Context::try_from(data.as_str())
}

fn main() {
    let args = Args::parse();
    let policy = match load_policy(args.policy.as_str()) {
        Ok(policy) => policy,
        Err(err) => {
            println!("Policy parse error: {:?}", err);
            return;
        }
    };
    let config = match RunConfig::try_from(&args) {
        Ok(config) => config,
        Err(err) => {
            println!("Argument error: {:?}", &err);
            return;
        }
    };

    match &config {
        RunConfig::None => println!("Policy successfully parsed"),
        RunConfig::Identity(action, resource, _context) => {
            match config.check(&policy) {
                Ok(result) => println!("Checked {:?} on {:?}: {:?}", action, resource, &result),
                Err(err) => println!("Error checking {:?} on {:?}: {:?}", action, resource, &err),
            };
        }
        RunConfig::Resource(principal, action, resource, _context) => {
            match config.check(&policy) {
                Ok(result) => println!("Checked {:?} doing {:?} on {:?}: {:?}", principal, action, resource, &result),
                Err(err) => println!("Error checking {:?} doing {:?} on {:?}: {:?}", principal, action, resource, &err),
            };
        }
    };
}
