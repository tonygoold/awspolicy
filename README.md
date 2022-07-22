`awspolicy` is an offline tool for parsing and evaluating AWS IAM policies.

This tool is a work in progress and currently incomplete.

# Running

You can build the tool using `cargo build` and run the resulting binary from the `target` build directory.
Alternatively, you can run directly using `cargo run -- <arguments>`. There are example policies in the `testdata` directory.

The tool supports the following arguments:

* `--policy <POLICY>`: A path to a policy JSON file. This must be provided exactly once.
* `--action <ACTION>`: Provide an AWS action (e.g., `iam:ChangePassword`) to evaluate against the policy. If provided, you must also provided a `--resource` argument.
* `--resource <RESOURCE>`: Provide an AWS resource (e.g., `arn:aws:iam::123456789012:user/Username`) to evaluate against the policy. If provided, you must also provide an `--action` argument.
* `--principal-aws <ARN>`: Provide an AWS principal as an ARN (e.g., `arn:aws:iam::123456789012:role/S3Access`) to evaluate against the policy. At most one principal can be provided.
* `--principal-canonical-user <USERID>`: Provide an AWS principal as a canonical user ID (e.g., `79a59df900b949e55d96a1e698fbacedfd6e09d98eacf8f8d5218e7cd47ef2be`) to evaluate against the policy. At most one principal can be provided.
* `--principal-federated <FEDERATION>`: Provide a web identity session principal or SAML session principal as a federated identifier (e.g., `accounts.google.com`) to evaluate against the policy. At most one principal can be provided.
* `--principal-service <SERVICE>`: Provide an AWS service principal as a service name (e.g., `ecs.amazonaws.com`) to evaluate against the policy. At most one principal can be provided.

If you provide a `--policy` argument and nothing else, then the tool parses the policy, prints a message if parsing was successful, and exits.

If you do not provide any principal argument, the policy is assumed to be an identity policy, and any Principal constraints in the policy are ignored. This may result in an error in a future iteration.

# To Do

An incomplete list of remaining work for the first version.

* Implement `...IfExists` condition operators.
* Implement `ForAllValues:...` and `ForAnyValues:...` condition operators.
* Simulate request context values (e.g., `aws:CurrentTime`).
* Implement policy variables.
* Add a mechanism for specifying policy variables (e.g., `arn:aws:iam::123456789012:user/${aws:username}`) in the evaluation context.
* Allow multiple policies to be provided for a single evaluation.
* Report meaningful errors.
* If a principal cannot directly perform an action, check whether the policy allows them to assume a role which can perform that action.
