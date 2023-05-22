## Rust EDI 

The following repo implements a simple EDI service. The suggested flow is as follows:

1. There are some queues. 
2. Producer(s) push messages to a queue/queues. 
3. Consumer(s) get messages from a queue/queues. 
4. Consumer(s) acknowledge message and remove them from a queue/queues. 

## Performance 
This is only a toy implementation, but being Rust, it works at baremetal speed. Sending, Receiving and Deleting 1000 messages using the Python implementation below takes a mere `2.2` seconds. 

There are many improvements that could be made. Here are a few: 
- Enable batch sends / returns to limit the # of requests and responses 
- Use web sockets for consumer connections to avoid more than a single request 

The limitation of requests will be important when authentication is added as checking credentials in each request is undoubtedly expensive. 

## Development 
The project is set up on the wonderful actix-web framework. The main entry point is in `src/main.rs`. `*_api.rs` implement the respective endpoints for a service. For example, `message_api.rs` implements all the endpoints for `message` specific actions. 

## Running 
Simply clone the repo and using your terminal run `cargo run`. 

## Current Endpoints 

- POST `/queue/new`: creates a new queue 
   - Request Body
    ```json 
    {
        "readTimeout": number, 
        "queueId": string
    }
    ```
   - Response 
    ```json 
    {
        "data": a string with the new queue's uuid, 
        "error": an error if any 
    }
    ```
- GET `/queue/list`: lists all queues 
    - Response
    ```json 
    {
        "data": a list of queue ids, 
        "error": an error if any 
    }
    ```
- POST `/message/new`: adds a message to a specified queue 
    - Request Body 
    ```json 
    {
        queueId: string, 
        messageId: string,
        content: string 
    }
    ```
    - Response 
    ```json 
    {
        "data": string representing the new messages uuid,
        "error": an error if any 
    }
    ```
- GET `/message/get`: gets a message 
    - Response 
    ```json 
    {
        "data" : {
                "messageId": string,
                "content": string,
                "uuid": string
            },
        "error": an eror if any 
    }
    ```
- POST `/message/delete`: deletes a message from a queue 
    - Request Body 
    ```json 
    {
        "queueId": string, 
        "messageUuid": string
    }
    ``` 
    - Response 
    ```json 
    {
        "data": a success message, 
        "error": an error if any  
    }
    ```
    **NOTE: Messages can only be deleted from the queue while their read timeout is in effect. Otherwise, they must be read again and deleted within their read timeout.**

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
        publish(queue_id, f"Message to You {i}", json.dumps({"Hello": "World"}))
        consume_and_delete(queue_id)

if __name__ == '__main__':
    main()
```