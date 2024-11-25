FROM node:18-alpine

WORKDIR /home/node/app
COPY components/fake-internet/package*.json ./
RUN npm install
COPY components/fake-internet .
ENV PORT 80
EXPOSE 80
CMD ["node", "index.js"]