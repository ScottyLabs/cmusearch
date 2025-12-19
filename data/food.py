import requests
import dotenv
import os
from minio import Minio
import io

dotenv.load_dotenv()

bucket_name = "cmusearch"

access_key = os.getenv("S3_ACCESS_KEY")
secret_key = os.getenv("S3_SECRET_KEY")
s3_endpoint = os.getenv("S3_ENDPOINT")

# Create client with access and secret key.
client = Minio(
    s3_endpoint,
    access_key=access_key,
    secret_key=secret_key,
)

response = requests.get("https://dining.apis.scottylabs.org/locations")
data = response.text.encode("utf-8")

client.put_object(
    bucket_name,
    "food.json",
    io.BytesIO(data),
    length=len(data),
    content_type="application/json",
)

# Create the json directory if it doesn't exist
os.makedirs("json", exist_ok=True)

with open("json/food.json", "w") as f:
    f.write(response.text)
