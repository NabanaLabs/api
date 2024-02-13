# Nabana Labs API
~~~py
HOST=0.0.0.0                            # Not Sensitive Data (fly.toml)
PORT=8080                               # Not Sensitive Data (fly.toml)

API_URL=                                # Not Sensitive Data (fly.toml)

POSTGRES_URI=                           # (optional) fly secrets set POSTGRES_URI=
MONGO_URI=                              # fly secrets set MONGO_URI=
REDIS_URI=                              # fly secrets set REDIS_URI=
MONGO_DB_NAME=                          # Not Sensitive Data (fly.toml)

API_TOKENS_SIGNING_KEY=                 # fly secrets set API_TOKENS_SIGNING_KEY=
API_TOKENS_EXPIRATION_TIME=

LEMONSQUEEZY_WEBHOOK_SIGNATURE_KEY=     # fly secrets set LEMONSQUEEZY_WEBHOOK_SIGNATURE_KEY=
PRO_PRODUCT_ID=                         # Not Sensitive Data (fly.toml)
PRO_MONTHLY_VARIANT_ID=                 # Not Sensitive Data (fly.toml)
PRO_ANNUALLY_VARIANT_ID=                # Not Sensitive Data (fly.toml)

ENABLE_EMAIL_VERIFICATION=              # Not Sensitive Data (fly.toml)

BREVO_CUSTOMERS_WEBFLOW_API_KEY=        # fly secrets set BREVO_CUSTOMERS_WEBFLOW_API_KEY=
BREVO_CUSTOMERS_LIST_ID=                # Not Sensitive Data (fly.toml)
BREVO_EMAIL_VERIFY_TEMPLATE_ID=         # Not Sensitive Data (fly.toml)

BREVO_MASTER_EMAIL_ADDRESS=             # Not Sensitive Data (fly.toml)
BREVO_MASTER_NAME=                      # Not Sensitive Data (fly.toml)

GOOGLE_OAUTH_CLIENT_ID=                 # Not Sensitive Data (fly.toml)
GOOGLE_OAUTH_CLIENT_SECRET=             # fly secrets set GOOGLE_OAUTH_CLIENT_SECRET= 
GOOGLE_OAUTH_CLIENT_REDIRECT_ENDPOINT=  # Not Sensitive Data (fly.toml)

PROMPT_CLASSIFICATION_MODEL_NAME=       # Not Sensitive Data (fly.toml)
PROMPT_CLASSIFICATION_MODEL_URL=        # Not Sensitive Data (fly.toml)
~~~