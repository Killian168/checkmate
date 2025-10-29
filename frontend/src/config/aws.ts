// AWS Configuration
export const awsConfig = {
  region: "eu-west-1",
  userPoolId:
    process.env.NEXT_PUBLIC_USER_POOL_ID || "REPLACE_WITH_USER_POOL_ID",
  userPoolWebClientId:
    process.env.NEXT_PUBLIC_USER_POOL_CLIENT_ID ||
    "REPLACE_WITH_USER_POOL_CLIENT_ID",
  oauth: {
    domain: process.env.NEXT_PUBLIC_OAUTH_DOMAIN || "REPLACE_WITH_OAUTH_DOMAIN",
    scopes: ["email", "openid"],
    redirectSignIn: [
      process.env.NEXT_PUBLIC_REDIRECT_SIGN_IN || "http://localhost:3000/",
    ],
    redirectSignOut: [
      process.env.NEXT_PUBLIC_REDIRECT_SIGN_OUT || "http://localhost:3000/",
    ],
    responseType: "code" as const,
  },
};

// API Configuration
export const apiConfig = {
  baseUrl:
    process.env.NEXT_PUBLIC_API_URL ||
    "https://REPLACE_WITH_API_ID.execute-api.eu-west-1.amazonaws.com/dev",
};
