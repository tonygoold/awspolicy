mod aws;
mod iam;
mod policy;

use aws::ARN;
use iam::{PrincipalKind, Principal, Action};
use policy::sample_policy;

fn main() {
    if let Ok(arn) = ARN::try_from("arn:aws:iam::123456789012:user/Username") {
        let user = Principal{arn, kind: PrincipalKind::User};
        println!("User: {}", &user);
    }
    if let Ok(arn) = ARN::try_from("arn:aws:iam::872449281171:role/aws-service-role/globalaccelerator.amazonaws.com/AWSServiceRoleForGlobalAccelerator") {
        let role = Principal{arn, kind: PrincipalKind::Role};
        println!("Role: {}", &role);
    }
    if let Ok(arn) = ARN::try_from("arn:aws:ses:ca-central-1:872449281171:identity/timberlea.net") {
        println!(
            "SES domain identity: service={}, region={}, account={}, resource={}",
            arn.service(),
            arn.region(),
            arn.account(),
            arn.resource(),
        );
    }

    if let Ok(action) = Action::try_from("route53:ListHostedZones") {
        println!("Action: {}", &action);
        println!("List hosted zones: service={}, action={}", action.service(), action.action());
    }

    match sample_policy() {
        Ok(policy) => {
            println!("{:?}", &policy);
            let action = Action::new("route53", "GetChange");
            let resource = ARN::new("route53", "", "", "change/Foo");
            let result = policy.check(&action, &resource);
            println!("Checked {:?} on {:?}: {:?}", &action, &resource, result);
        }
        Err(err) => println!("Policy parse error: {:?}", err),
    }
}
