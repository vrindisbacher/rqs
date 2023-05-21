## Rust EDI 

The following repo implements a simple EDI service. The suggested flow is as follows:

1. There are 1 or more queues. 
2. Producer(s) push messages to a queue. 
3. Consumer(s) get messages from a queue. 
4. Consumer(s) acknowledge message and remove them from a queue. 

## Development 
The project is set up on the wonderful actix-web framework. The main entry point is in `src/main.rs`. `*_api.rs` implement the respective endpoints for a service. For example, `message_api.rs` implements all the endpoints for `message` specific actions. 

## Running 
Simply clone the repo and using your terminal run `cargo run`. 

## Example 
Here is some python that loosely sends and receives messages using all current features. 

```python 
import requests 
import json 

def create_queue(queue_id):
    requests.post("http://127.0.0.1:8080/queue/new", json={"readTimeout": 10, "queueId": queue_id})

def list_queues():
    requests.get("http://127.0.0.1:8080/queue/list")

def publish(queue_id, message_id, message_content):
    post_data = {
        "queueId": queue_id,
        "messageId": message_id,
        "content": message_content
    }
    r = requests.post("http://127.0.0.1:8080/message/new", json=post_data)
    print(f"produced {r.json()}")

def delete_message(queue_id, message_uuid): 
    post_data = {
        "queueId": queue_id,
        "messageUuid": message_uuid
    }
    requests.post("http://127.0.0.1:8080/message/new", json=post_data)

def consume_and_delete(queue_id):
    r = requests.get(f"http://127.0.0.1:8080/message/get?queueId={queue_id}")
    message = r.json()
    print(f"received {message}")
    uuid = message["data"]["uuid"]
    post_data = {
        "queueId": queue_id, 
        "messageUuid": uuid 
    }
    r = requests.post("http://127.0.0.1:8080/message/delete", json=post_data)

def main(): 
    queue_id = 'my-test-queue'
    create_queue(queue_id)
    list_queues()

    for i in range(100):
        publish(queue_id, f"Message to Emily {i}", json.dumps({"Emily": "Is a Nerd"}))
        consume_and_delete(queue_id)

if __name__ == '__main__':
    main()
```