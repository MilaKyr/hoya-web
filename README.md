# hoya-web

Run local postgres database:
```
cd webapp
docker build -t postgres ./
docker run --net=host --name local-postgres \
    -e POSTGRES_PASSWORD=password -e POSTGRES_USER=main -d postgres
```
