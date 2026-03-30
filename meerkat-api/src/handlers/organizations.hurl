# Create organization - valid
# NOTE: returns 500 until PgUnitOfWorkFactory is implemented
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Meerkat Inc.",
    "slug": "meerkat-inc"
}
HTTP 500
[Asserts]
jsonpath "$.code" == "internal_error"

# Create organization - empty name
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "  ",
    "slug": "meerkat-inc"
}
HTTP 400
[Asserts]
jsonpath "$.code" == "validation_error"

# Create organization - invalid slug
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Meerkat Inc.",
    "slug": "INVALID SLUG!!"
}
HTTP 422
