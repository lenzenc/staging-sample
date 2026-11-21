from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    app_env: str = "local"
    mongodb_uri: str = "mongodb://localhost:27017"
    mongodb_db: str = "liabilities"


settings = Settings()
