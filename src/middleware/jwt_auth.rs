use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            println!("-> middleware/jwt_auth.rs -  token: {:?}", token);
            if let Some(token) = token {
                println!("AuthMiddleware received token: [{}]", token); // 실제 토큰 값 출
                match crate::utils::jwt::verify_jwt(&token) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        let res = service.call(req).await?;
                        Ok(res.map_into_left_body())
                    }
                    Err(_) => {
                        let response = req.into_response(
                            HttpResponse::Unauthorized()
                                .body("Invalid token")
                                .map_into_right_body(),
                        );
                        Ok(response)
                    }
                }
            } else {
                let response = req.into_response(
                    HttpResponse::Unauthorized()
                        .body("Missing token")
                        .map_into_right_body(),
                );
                Ok(response)
            }
        })
    }
}
