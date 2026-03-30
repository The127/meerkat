# Liveness check
GET {{host}}/health
HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"

# Readiness check
GET {{host}}/health/ready
HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"
