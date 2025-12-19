import dotenv
import os
from minio import Minio
import json
import io

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

# Fetch the buildings data from the S3 bucket
response = client.get_object("cmumaps", "floorplans/buildings.json")
building_data = json.loads(response.read().decode("utf-8"))


# Process the buildings data
new_building_data = dict()
for building_code in building_data:
    building = building_data[building_code]
    new_building_data[building_code] = {
        "nameWithSpace": building_code,
        "fullNameWithSpace": building["name"],
        "id": building_code,
        "type": "Building",
        "labelPosition": building["labelPosition"],
        "alias": building["name"],
        # "numTerms":
        # "floor": {
        #     "buildingCode": building_code,
        #     "level": "1"
        # }
    }

# Create the json directory if it doesn't exist
os.makedirs("json", exist_ok=True)

# Write the processed buildings data to a file
with open("json/buildings.json", "w") as f:
    json.dump(new_building_data, f)

# Upload the processed buildings data to the S3 bucket
data = json.dumps(new_building_data).encode("utf-8")
client.put_object(
    "cmusearch",
    "buildings.json",
    io.BytesIO(data),
    length=len(data),
    content_type="application/json",
)
