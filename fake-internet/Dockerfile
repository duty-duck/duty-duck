FROM node:18-alpine

WORKDIR /home/node/app
COPY fake-internet/package*.json ./
RUN npm install
COPY fake-internet .
ENV PORT 80
EXPOSE 80
CMD ["node", "index.js"]