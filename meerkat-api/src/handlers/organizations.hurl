# Create organization - valid
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Meerkat Inc.",
    "slug": "meerkat-inc",
    "oidc_config": {
        "name": "Default SSO",
        "client_id": "meerkat-client",
        "issuer_url": "https://auth.example.com",
        "audience": "meerkat-api",
        "claim_mapping": {
            "sub_claim": "sub",
            "name_claim": "preferred_username",
            "role_claim": "roles",
            "role_values": {
                "owner": ["owner"],
                "admin": ["admin"],
                "member": ["member"]
            }
        }
    }
}
HTTP 201
[Asserts]
jsonpath "$.id" isString

# Create organization - empty name
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "  ",
    "slug": "some-slug",
    "oidc_config": {
        "name": "Default SSO",
        "client_id": "meerkat-client",
        "issuer_url": "https://auth.example.com",
        "audience": "meerkat-api",
        "claim_mapping": {
            "sub_claim": "sub",
            "name_claim": "preferred_username",
            "role_claim": "roles",
            "role_values": {
                "owner": ["owner"],
                "admin": ["admin"],
                "member": ["member"]
            }
        }
    }
}
HTTP 400
[Asserts]
jsonpath "$.code" == "validation_error"

# Create organization - invalid slug
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Meerkat Inc.",
    "slug": "INVALID SLUG!!",
    "oidc_config": {
        "name": "Default SSO",
        "client_id": "meerkat-client",
        "issuer_url": "https://auth.example.com",
        "audience": "meerkat-api",
        "claim_mapping": {
            "sub_claim": "sub",
            "name_claim": "preferred_username",
            "role_claim": "roles",
            "role_values": {
                "owner": ["owner"],
                "admin": ["admin"],
                "member": ["member"]
            }
        }
    }
}
HTTP 422
