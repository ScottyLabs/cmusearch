import dotenv
import os
from minio import Minio
import json

dotenv.load_dotenv()

access_key = os.getenv("S3_ACCESS_KEY")
secret_key = os.getenv("S3_SECRET_KEY")
s3_endpoint = os.getenv("S3_ENDPOINT")

# Create client with access and secret key.
client = Minio(
    s3_endpoint,
    access_key=access_key,
    secret_key=secret_key,
)

# Fetch the 25live data from the S3 bucket
response = client.get_object("event-scraper", "25live.json")
twentyfive_live_data = json.loads(response.read().decode("utf-8"))
with open("json/25live.json", "w") as f:
    json.dump(twentyfive_live_data, f)

# Fetch the Handshake data from the S3 bucket
response = client.get_object("event-scraper", "handshake.json")
handshake_data = json.loads(response.read().decode("utf-8"))
with open("json/handshake.json", "w") as f:
    json.dump(handshake_data, f)

# Fetch the Tartan Connect data from the S3 bucket
response = client.get_object("event-scraper", "tartan-connect.json")
tartan_connect_data = json.loads(response.read().decode("utf-8"))
with open("json/tartan-connect.json", "w") as f:
    json.dump(tartan_connect_data, f)
