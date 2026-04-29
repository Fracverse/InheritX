# GraphQL API

The InheritX backend now includes a GraphQL API to provide more flexibility to frontend consumers alongside the existing REST API.

## Endpoints

- **GraphQL API:** `POST /api/graphql`
- **Playground:** `GET /api/graphql/playground`

## Overview

The GraphQL schema provides queries for key entities such as `Plan` and `UserReputation`.

### Example Queries

#### Fetch a Plan
```graphql
query GetPlan($id: String!) {
  plan(id: $id) {
    id
    userId
    status
  }
}
```

#### Fetch User Reputation
```graphql
query GetReputation($userId: String!) {
  reputation(userId: $userId) {
    userId
    score
    totalLoansTaken
    totalLoansRepaid
  }
}
```

## Using the Playground

You can explore the schema and test queries interactively by visiting `/api/graphql/playground` in your browser while the server is running.
