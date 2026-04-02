# Create an organization first (FK dependency)
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Test Org",
    "slug": "test-org-for-projects",
    "oidc_config": {
        "name": "Default SSO",
        "client_id": "meerkat-client",
        "issuer_url": "https://auth.example.com",
        "audience": "meerkat-api",
        "claim_mapping": {
            "sub_claim": "sub",
            "name_claim": "preferred_username",
            "role_claim": "roles",
            "owner_values": ["owner"],
            "admin_values": ["admin"],
            "member_values": ["member"]
        }
    }
}
HTTP 201
[Captures]
org_id: jsonpath "$.id"

# Create project - valid
POST {{host}}/api/v1/projects
Content-Type: application/json
{
    "organization_id": "{{org_id}}",
    "name": "My Project",
    "slug": "my-project"
}
HTTP 201
[Asserts]
jsonpath "$.id" isString

# Create project - empty name
POST {{host}}/api/v1/projects
Content-Type: application/json
{
    "organization_id": "{{org_id}}",
    "name": "  ",
    "slug": "some-slug"
}
HTTP 400
[Asserts]
jsonpath "$.code" == "validation_error"

# Create project - invalid slug
POST {{host}}/api/v1/projects
Content-Type: application/json
{
    "organization_id": "{{org_id}}",
    "name": "My Project",
    "slug": "INVALID SLUG!!"
}
HTTP 422
