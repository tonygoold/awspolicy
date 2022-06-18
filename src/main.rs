mod aws;
mod iam;
mod policy;

use aws::ARN;
use iam::Action;
use policy::sample_policy;

fn main() {
    match sample_policy() {
        Ok(policy) => {
            println!("{:?}\n", &policy);
            let action = Action::new("route53", "GetChange");
            let resource = ARN::new("route53", "", "", "change/Foo");
            let result = policy.check(&action, &resource);
            println!("Checked {:?} on {:?}: {:?}", &action, &resource, result);
        }
        Err(err) => println!("Policy parse error: {:?}", err),
    }
}
