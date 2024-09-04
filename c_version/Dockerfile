FROM gcc:latest AS builder

WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN make

FROM ubuntu:latest

WORKDIR /app

# Copy the built application from the builder stage
COPY --from=builder /app/cmadv /app/cmadv

# Set the entrypoint
ENTRYPOINT ["/app/cmadv"]

