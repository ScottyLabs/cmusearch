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

# Construct building code to name
response = client.get_object("cmumaps", "floorplans/buildings.json")
building_data = json.loads(response.read().decode("utf-8"))
building_code_to_name = {"outside": "Outside"}
for building_code in building_data:
    building = building_data[building_code]
    building_code_to_name[building_code] = building["name"]

# Fetch the buildings data from the S3 bucket
response = client.get_object("cmumaps", "floorplans/floorplans.json")
rooms_data = json.loads(response.read().decode("utf-8"))

# Process the floorplans data
new_rooms_data = dict()
for building_code in rooms_data:
    building = rooms_data[building_code]
    for floor_code in building:
        floor = building[floor_code]
        for room_id in floor:
            room = floor[room_id]
            new_rooms_data[room_id] = {
                "nameWithSpace": f"{building_code} {room['name']}",
                "fullNameWithSpace": f"{building_code_to_name[building_code]} {room['name']}",
                "id": room_id,
                "labelPosition": room["labelPosition"],
                "type": room["type"],
                "floor": {"buildingCode": building_code, "level": floor_code},
                "aliases": room["aliases"],
                # "numTerms":
            }

# Write the processed buildings data to a file
with open("json/rooms.json", "w") as f:
    json.dump(new_rooms_data, f)

# Upload the processed buildings data to the S3 bucket
data = json.dumps(new_rooms_data).encode("utf-8")
client.put_object(
    "cmusearch",
    "rooms.json",
    io.BytesIO(data),
    length=len(data),
    content_type="application/json",
)
