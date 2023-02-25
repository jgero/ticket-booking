# Ticket booking service

This service is supposed to solve the issue of [ordering tickets for events of a
local club](doc/use_cases.md).

More information about the internal architecture can be found
[here](doc/architecture.md).

## Development

- [rust-rdkafka](https://github.com/fede1024/rust-rdkafka)

Run local Kafka instance with `podman kube play` or Kubernetes

## Testing

- `curl --header "Content-Type: application/json"   --request POST --data '{"ticket_visible_ids": ["WED-001","WED-002"]}' http://localhost:8000/order`
- `kafka-console-consumer.sh --bootstrap-server 127.0.0.1:9092 --topic placed-orders --from-beginning`

## Ideas

- read ticket visible ids and layouts from a SVG. This could double as
  configuration and graphic to show in the web interface
- 'spare' tickets: tickets which only can be ordered once a threshold of the
  total amount of tickets are ordered

