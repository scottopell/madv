# Start with a golang base image to build the application
FROM golang:1.23 AS builder

WORKDIR /app
COPY . .

# Build the Go app
RUN CGO_ENABLED=0 go build -o gomadv main.go

FROM ubuntu:latest

WORKDIR /root/

COPY --from=builder /app/gomadv .

CMD ["./gomadv"]

