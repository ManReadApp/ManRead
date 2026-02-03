FROM node:18

WORKDIR /app
COPY frontend/ .
RUN rm -rf node_modules
RUN npm install

CMD ["npm", "run", "preview"]
