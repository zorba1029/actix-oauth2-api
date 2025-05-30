#!/bin/bash

#--------------------------------
# Register user test on MongoDB
#--------------------------------
curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d '{"username":"djamware","email":"admin@djamware.com","password":"mypassword"}'
#-- return
# User registered successfully
# Email already exists

#--------------------------------
# Login user test on MongoDB
#--------------------------------
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@djamware.com", "password": "mypassword"}'
#-- return
# {
# "access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg2MTIzMDgsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.EsdOrA5pjOBfQGYDnkTw28XH4v_QxHvQHy-sdQtlRIY",
# "refresh_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkyMTYyMDgsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.JjrHtOX5E1fl_Kgi75zENBgxVaT_lDlt_FusSbV-tMM"
# }

#-- 5/29/2025
> curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@djamware.com", "password": "mypassword"}' | jq
#-- return
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg1NjQ3MzgsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.fC_Qw2t5f3We_y5dKPVrHcQKRvHAc-VrkhUX54RgbrA",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjg2MzgsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.l5U9vk_fBtuJUB0kCXixHpVTyzb8Ts7ehNi0DlkFWSk"
}

#--------------------------------
# Get profile test on MongoDB
#--------------------------------
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg1NjQ3MzgsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.fC_Qw2t5f3We_y5dKPVrHcQKRvHAc-VrkhUX54RgbrA" \
  http://localhost:8080/api/profile
#-- return
# {"email":"admin@djamware.com","message":"Your are authorized. This is a protected route"}% 

curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg2MTEyMTMsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.UTI1_X2ZoXqRTd8pHJhJ2YW9VI-avlGI64azR5JqAi0" \
   http://localhost:8080/api/profile | jq
#-- return
# {
#   "email": "admin@djamware.com",
#   "message": "Your are authorized. This is a protected route"
# }

curl -X POST http://localhost:8080/refresh \
  -H "Content-Type: application/json" \
  -d '{ "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjkyODYsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.UvOgyOh93fAGStaMtvsBf67UBrno8XRC0rInOS8h42o" }'
#-- return
# {
#   "access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg1NjU0MTAsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.eLj5kg2kGJINa1Ss2Lt4wqsyh6pBn9PYjHqOnft-sK4",
#   "refresh_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjkzMTAsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.XPX_FJoRnTaP0hRi2xecGAM6r63lFdnpgbo7gdnTN_I"
# }

curl -X POST http://localhost:8080/refresh \
  -H "Content-Type: application/json" \
  -d '{ "refresh_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjk5NzQsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.vL0jL-a1JMOJg6XaccSVnHMk1V9iXMGpnb-3rk12IuM"}' | jq
# {
#   "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDg1NjU3MTIsInRva2VuX3R5cGUiOiJhY2Nlc3MifQ.1EQaAIPF5iXHeYA9GbMjEJwXhvwU9UrZPhgef2P7wdg",
#   "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjk2MTIsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.C0B9bvv1VYbnzMfC7yCjVGSHJ4ighaXPDXAS4c8W6aU"
# }

#--------------------------------
# Logout user test on MongoDB
#--------------------------------
curl -X POST http://localhost:8080/logout \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBkamFtd2FyZS5jb20iLCJleHAiOjE3NDkxNjk2MTIsInRva2VuX3R5cGUiOiJyZWZyZXNoIn0.C0B9bvv1VYbnzMfC7yCjVGSHJ4ighaXPDXAS4c8W6aU"

#-- return
# Logged out successfully