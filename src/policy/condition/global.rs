// A list of global keys with types and cardinality
// All these keys have a "aws:" prefix.

pub enum Type {
	String,
	Numeric,
	Date,
	Epoch, // Supports both Date and Numeric operators
	Bool,
	Binary,
	IpAddress,
	ARN,
	UnknownType,
}

pub enum Cardinality {
	Optional,
	Required,
	Multiple,
	UnknownCardinality,
}

use Type::*;
use Cardinality::*;

// See: https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_condition-keys.html
pub const AWS: &[(&str, Type, Cardinality)] = &[
	("CalledVia", String, Multiple),
	("CalledViaFirst", String, Optional),
	("CalledViaLast", String, Optional),
	("CurrentTime", Date, Required),
	("EpochTime", Epoch, Required),
	("FederatedProvider", String, Optional),
	("MultiFactorAuthAge", Numeric, Optional),
	("MultiFactorAuthPresent", Bool, Optional),
	("PrincipalAccount", String, Required),
	("PrincipalArn", ARN, Optional),
	("PrincipalIsAWSService", Bool, Optional),
	("PrincipalOrgID", String, Optional),
	("PrincipalOrgPaths", String, Multiple),
	("PrincipalServiceName", String, Optional),
	("PrincipalServiceNamesList", String, Multiple),
	// Used in the form aws:PrincipalTag/tag-key
	("PrincipalTag", String, Optional),
	("PrincipalType", String, Required),
	// Uses lowercase aws:referer
	("Referer", String, Optional),
	("RequestedRegion", String, Required),
	// Used in the form aws:RequestTag/tag-key
	("RequestTag", String, Optional),
	// Some actions do not support this key, but it is always present for those that support it
	("ResourceAccount", String, Required),
	("ResourceOrgID", String, Optional),
	("ResourceOrgPaths", String, Multiple),
	// Used in the form aws:ResourceTag/tag-key
	("ResourceTag", String, Optional),
	("SecureTransport", Bool, Required),
	("SourceAccount", String, Optional),
	("SourceArn", ARN, Optional),
	("SourceIdentity", String, Optional),
	("SourceIp", IpAddress, Optional),
	("SourceVpc", String, Optional),
	("SourceVpce", String, Optional),
	("TagKeys", String, Multiple),
	("TokenIssueTime", Date, Optional),
	("UserAgent", String, Required),
	// Uses lowercase aws:userid
	("Userid", String, Required),
	// Uses lowercase aws:username
	("Username", String, Optional),
	("ViaAWSService", Bool, Required),
	("VpcSourceIp", IpAddress, Optional),
];
