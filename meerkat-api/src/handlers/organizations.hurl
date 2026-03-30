# Create organization - valid
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "Meerkat Inc.",
    "slug": "meerkat-inc"
}
HTTP 201
[Asserts]
jsonpath "$.id" isString

# Create organization - empty name
POST {{host}}/api/v1/organizations
Content-Type: application/json
{
    "name": "  ",
    "slug": "some-slug"
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
