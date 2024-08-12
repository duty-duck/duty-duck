# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-frontend:latest -f frontend/Dockerfile . 
FROM node:18-alpine as builder

RUN mkdir -p /usr/src/dutyduck/frontend
WORKDIR /usr/src/dutyduck/frontend

RUN apk update && apk upgrade

COPY ./server/bindings ../server/bindings
COPY ./frontend .
RUN npm install
RUN npm run build

CMD [ "npm", "start" ]