FROM node:18-alpine

WORKDIR /app
COPY package-lock.json package.json /app/
COPY components/fake-internet/package*.json /app/components/fake-internet/
RUN npm install -w components/fake-internet --production
COPY components/fake-internet /app/components/fake-internet
ENV PORT 80
EXPOSE 80
CMD ["node", "/app/components/fake-internet/index.js"]