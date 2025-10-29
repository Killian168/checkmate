import { Amplify } from "aws-amplify";
import { awsConfig } from "./aws";

// Global configuration state
let amplifyConfigured = false;

// Validate that environment variables are properly set
if (awsConfig.userPoolId === "REPLACE_WITH_USER_POOL_ID") {
  throw new Error(
    "NEXT_PUBLIC_USER_POOL_ID environment variable is not set. Please update your .env.local file with the correct User Pool ID.",
  );
}

if (awsConfig.userPoolWebClientId === "REPLACE_WITH_USER_POOL_CLIENT_ID") {
  throw new Error(
    "NEXT_PUBLIC_USER_POOL_CLIENT_ID environment variable is not set. Please update your .env.local file with the correct User Pool Client ID.",
  );
}

// Validate User Pool ID format
if (awsConfig.userPoolId && !awsConfig.userPoolId.includes("_")) {
  console.warn(
    "User Pool ID format may be incorrect. Expected format: region_ID (e.g., eu-west-1_XXXXXXXXX)",
  );
}

// Validate User Pool Client ID length
if (
  awsConfig.userPoolWebClientId &&
  awsConfig.userPoolWebClientId.length !== 26
) {
  console.warn(
    "User Pool Client ID length seems incorrect. Cognito client IDs are typically 26 characters long.",
  );
}

// OAuth domain is optional; comment out if not needed
// if (awsConfig.oauth.domain === "REPLACE_WITH_OAUTH_DOMAIN") {
//   throw new Error(
//     "NEXT_PUBLIC_OAUTH_DOMAIN environment variable is not set. Please update your .env.local file with the correct OAuth Domain.",
//   );
// }

try {
  const configToApply = {
    Auth: {
      Cognito: {
        region: awsConfig.region,
        userPoolId: awsConfig.userPoolId,
        userPoolClientId: awsConfig.userPoolWebClientId,
        loginWith: {
          email: true,
        },
      },
    },
  };

  Amplify.configure(configToApply);
  amplifyConfigured = true;
} catch (error) {
  console.error("Failed to configure Amplify:", error);
  throw error;
}

export { amplifyConfigured };
export default Amplify;
