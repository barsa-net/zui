ARG NODE_VERSION=16
ARG NGINX_VERSION=stable

FROM docker.io/library/node:$NODE_VERSION AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

RUN npm run build

FROM docker.io/library/nginx:$NGINX_VERSION

COPY --from=builder /app/build /usr/share/nginx/html