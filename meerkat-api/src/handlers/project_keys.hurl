# Create an organization first (FK dependency)
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Key Test Org",
    "slug": "key-test-org",
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

# Create a project (auto-generates one default key)
POST {{host}}/api/v1/projects
Content-Type: application/json
Host: key-test-org.127.0.0.1
{
    "organization_id": "{{org_id}}",
    "name": "Key Test Project",
    "slug": "key-test-project"
}
HTTP 201

# List keys — should have 1 auto-generated key
GET {{host}}/api/v1/projects/key-test-project/keys
Host: key-test-org.127.0.0.1
HTTP 200
[Asserts]
jsonpath "$.total" == 1
jsonpath "$.items[0].label" == "Default"
jsonpath "$.items[0].status" == "active"
jsonpath "$.items[0].key_token" isString
jsonpath "$.items[0].dsn" isString
[Captures]
default_key_id: jsonpath "$.items[0].id"

# Create an additional key
POST {{host}}/api/v1/projects/key-test-project/keys
Content-Type: application/json
Host: key-test-org.127.0.0.1
{
    "label": "Production"
}
HTTP 201
[Asserts]
jsonpath "$.id" isString
[Captures]
new_key_id: jsonpath "$.id"

# List keys — should have 2
GET {{host}}/api/v1/projects/key-test-project/keys
Host: key-test-org.127.0.0.1
HTTP 200
[Asserts]
jsonpath "$.total" == 2

# Revoke the new key
DELETE {{host}}/api/v1/projects/key-test-project/keys/{{new_key_id}}
Host: key-test-org.127.0.0.1
HTTP 204

# List keys — still 2 but one is revoked
GET {{host}}/api/v1/projects/key-test-project/keys
Host: key-test-org.127.0.0.1
HTTP 200
[Asserts]
jsonpath "$.total" == 2

# Create key with empty label — should fail
POST {{host}}/api/v1/projects/key-test-project/keys
Content-Type: application/json
Host: key-test-org.127.0.0.1
{
    "label": "  "
}
HTTP 400
[Asserts]
jsonpath "$.code" == "validation_error"
