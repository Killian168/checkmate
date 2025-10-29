import { fetchAuthSession } from "aws-amplify/auth";
import { apiConfig } from "../config/aws";
import { User } from "../types/user";

export class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function getCurrentUser(): Promise<User> {
  try {
    const session = await fetchAuthSession();
    const token = session.tokens?.accessToken?.toString();

    if (!token) {
      throw new ApiError("No authentication token available");
    }

    const response = await fetch(`${apiConfig.baseUrl}/users/me`, {
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw new ApiError(
        `API request failed: ${response.status} ${response.statusText}`,
        response.status,
      );
    }

    return await response.json();
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(
      `Failed to fetch user data: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

export async function deleteCurrentUser(): Promise<void> {
  try {
    const session = await fetchAuthSession();
    const token = session.tokens?.accessToken?.toString();

    if (!token) {
      throw new ApiError("No authentication token available");
    }

    const response = await fetch(`${apiConfig.baseUrl}/users/me`, {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw new ApiError(
        `API request failed: ${response.status} ${response.statusText}`,
        response.status,
      );
    }
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(
      `Failed to delete user: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}
