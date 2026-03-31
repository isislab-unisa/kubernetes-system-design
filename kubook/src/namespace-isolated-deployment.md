# Namespace-isolated deployment

Design and deploy the same application with its internal Service into separate Namespaces to simulate staging and production environments.

This category includes the following learning objectives:
- Understanding of Pods.
- Understanding of Deployments.
- Understanding of ClusterIP Services.
- Understanding of Namespace isolation, resource scoping, and deploying objects into specific Namespaces.

## Task 1: Design and deploy a web application in staging and production namespaces

Your team needs to run the same internal web application in two isolated environments: `staging` and `production`. Each environment must be fully self-contained, with its own Deployment and Service, so that changes in one environment cannot affect the other.

The web application must run as a [hello-kubernetes](https://hub.docker.com/r/paulbouwer/hello-kubernetes) container, which displays the namespace it is running in, making it easy to confirm namespace isolation visually. It does not need to be highly resilient, since brief periods of unavailability are acceptable.

Other services within each namespace need a stable address to reach the web application, but it must not be accessible from outside the cluster.

### Architectural design

The task requires running the same application in two isolated environments, brief downtime is acceptable, and the application must be reachable only from inside each namespace. These constraints drive four design decisions:

1. Two separate Namespaces (`staging` and `production`) provide the isolation boundary. Every Kubernetes resource is scoped to a Namespace, so Deployments, Pods, and Services created in one Namespace are invisible to the other. This lets both environments share the same resource names without conflict.

2. Because the application is a single container and brief downtime is acceptable, a Deployment with one replica per Namespace is enough. Each Deployment creates its own ReplicaSet, which recreates the Pod automatically if it crashes, at the cost of a short period of unavailability that the task explicitly allows.

3. Other services within each Namespace need a stable address to reach the web application. Pod IPs change every time a Pod is recreated, so we place a ClusterIP Service (`web-app-svc`) in front of the Pod in each Namespace. The Service provides a fixed cluster-internal DNS name and forwards traffic to the Pod. It accepts requests on port `80` and forwards them to the container's port `8080`.

4. The application must not be accessible from outside the cluster. A ClusterIP Service has no external port and no route from outside the cluster network, so it satisfies this requirement by design. No Gateway, Ingress, or NodePort is needed.

![Architecture diagram](diagrams_images/namespace-isolated-deployment.png)

The diagram shows the resulting architecture: the `staging` and `production` Namespaces each contain an independent Deployment and ClusterIP Service with the same names. External clients have no path into either environment, while internal services reach the web application through the ClusterIP Service in their own Namespace. Cross-namespace access is possible only via the fully qualified DNS name (`web-app-svc.<namespace>.svc.cluster.local`), since short Service names resolve only within the same Namespace.

### Implementation

We start by creating the two namespaces:

```bash
kubectl create namespace staging
kubectl create namespace production
```

Next, we create a file called `web-app.yaml` that will be reused for both environments:

```bash
cat <<EOF > web-app.yaml
```

With the following content:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  labels:
    app: web-app
spec:
  replicas: 1
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      containers:
        - name: hello-kubernetes
          image: paulbouwer/hello-kubernetes:1.10
          ports:
            - containerPort: 8080
          env:
            - name: KUBERNETES_NAMESPACE
              valueFrom:
                fieldRef:
                  fieldPath: metadata.namespace
EOF
```

The `KUBERNETES_NAMESPACE` environment variable is injected using the downward API, which allows a container to read its own Pod metadata at runtime. The `hello-kubernetes` application uses this variable to display the namespace in its response.

Notice that the manifest does not include a `namespace` field in the metadata. We will supply the target namespace at apply time using the `-n` flag, which lets us reuse the same manifest for both environments.

To verify the file was created correctly, run:

```bash
cat web-app.yaml
```

Apply the manifest to both namespaces:

```bash
kubectl apply -f web-app.yaml -n staging
kubectl apply -f web-app.yaml -n production
```

Next, we expose each Deployment as a ClusterIP Service inside its respective namespace:

```bash
kubectl expose deployment web-app \
    -n staging \
    --name=web-app-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=8080
```

```bash
kubectl expose deployment web-app \
    -n production \
    --name=web-app-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=8080
```

#### Verify resource creation

To verify that the Pods are running in each namespace, execute the following commands:

```bash
kubectl get pods -n staging -l app=web-app
kubectl get pods -n production -l app=web-app
```

The output for each should look similar to this:

```bash
NAME                          READY   STATUS    RESTARTS   AGE
web-app-6bfbf8b67c-m4t9x      1/1     Running   0          1m
```

To verify that the Services are configured correctly in each namespace, run:

```bash
kubectl get svc -n staging web-app-svc
kubectl get svc -n production web-app-svc
```

The output for each should look similar to this:

```bash
NAME             TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)   AGE
web-app-svc      ClusterIP   10.96.112.54    <none>        80/TCP    1m
```

Note that the two Services share the same name (`web-app-svc`) but have different Cluster IPs, because they are independent resources in separate namespaces.

#### Test the web application

To test the staging web application, create a temporary Pod inside the `staging` namespace and send a request through the Service:

```bash
kubectl run -n staging -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside the busybox Pod, use `wget` to access the web application through the Service:

```bash
wget -qO- http://web-app-svc
```

The response should be the hello-kubernetes HTML page showing the namespace the Pod is running in:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Hello Kubernetes!</title>
    <!-- CSS styles omitted for brevity -->
</head>
<body>
  <div class="main">
    <!-- Content omitted for brevity -->
    <div class="content">
      <div id="message">Hello world!</div>
      <div id="info">
        <table>
          <tr>
            <th>namespace:</th>
            <td>staging</td>
          </tr>
          <tr>
            <th>pod:</th>
            <td>web-app-67d9bd9d5d-n5t7g</td>
          </tr>
          <tr>
            <th>node:</th>
            <td>- (Linux 6.8.0-101-generic)</td>
          </tr>
        </table>
      </div>
    </div>
  </div>
</body>
</html>
```

To confirm that the response contains the correct namespace, run:

```bash
wget -qO- http://web-app-svc | grep -A1 'namespace'
```

The output should show the `staging` namespace:

```html
<th>namespace:</th>
<td>staging</td>
```

Repeat the same test for the `production` namespace by running the busybox Pod with `-n production`. The grep output should show `production` instead of `staging`, confirming that each Deployment is running in its own isolated namespace.

#### Verify namespace isolation

To confirm that the short Service name does not resolve across namespaces, create a temporary Pod in the default namespace:

```bash
kubectl run -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside this Pod, attempt to reach the staging web application using its short service name:

```bash
wget -qO- --timeout=5 http://web-app-svc
```

This fails because short Service names only resolve within the same namespace. Services in other namespaces are reachable using their fully qualified DNS name (`<service>.<namespace>.svc.cluster.local`):

```bash
wget -qO- http://web-app-svc.staging.svc.cluster.local
```

This request succeeds, demonstrating that Kubernetes namespaces scope resource visibility and RBAC, but do not enforce network-level isolation on their own. To restrict cross-namespace traffic, NetworkPolicies must be used in addition to namespaces.

The same can be done to access the production web application:

```bash
wget -qO- http://web-app-svc.production.svc.cluster.local
```

## Task 2: Design and deploy an internal API status endpoint in dev and QA namespaces

Your team needs to run the same internal API status endpoint in two isolated environments: `dev` and `qa`. Each environment must be fully self-contained, with its own Deployment and Service, so that developers and testers can work independently without interfering with each other.

The API status endpoint must run as a [podinfo](https://hub.docker.com/r/stefanprodan/podinfo) container, which returns JSON metadata including a configurable message that displays the namespace it is running in, making it easy to confirm namespace isolation programmatically. It does not need to be highly resilient, since brief periods of unavailability are acceptable.

Other services within each namespace need a stable address to reach the API status endpoint, but it must not be accessible from outside the cluster.

### Architectural design

The task requires running the same application in two isolated environments, brief downtime is acceptable, and the application must be reachable only from inside each namespace. These constraints drive four design decisions:

1. Two separate Namespaces (`dev` and `qa`) provide the isolation boundary. Every Kubernetes resource is scoped to a Namespace, so Deployments, Pods, and Services created in one Namespace are invisible to the other. This lets both environments share the same resource names without conflict.

2. Because the application is a single container and brief downtime is acceptable, a Deployment with one replica per Namespace is enough. Each Deployment creates its own ReplicaSet, which recreates the Pod automatically if it crashes, at the cost of a short period of unavailability that the task explicitly allows.

3. Other services within each Namespace need a stable address to reach the API status endpoint. Pod IPs change every time a Pod is recreated, so we place a ClusterIP Service (`api-status-svc`) in front of the Pod in each Namespace. The Service provides a fixed cluster-internal DNS name and forwards traffic to the Pod. It accepts requests on port `80` and forwards them to the container's port `9898`.

4. The application must not be accessible from outside the cluster. A ClusterIP Service has no external port and no route from outside the cluster network, so it satisfies this requirement by design. No Gateway, Ingress, or NodePort is needed.

![Architecture diagram](diagrams_images/namespace-isolated-deployment_task2.png)

The diagram shows the resulting architecture: the `dev` and `qa` Namespaces each contain an independent Deployment and ClusterIP Service with the same names. External clients have no path into either environment, while internal services reach the API status endpoint through the ClusterIP Service in their own Namespace. Cross-namespace access is possible only via the fully qualified DNS name (`api-status-svc.<namespace>.svc.cluster.local`), since short Service names resolve only within the same Namespace.

### Implementation

We start by creating the two namespaces:

```bash
kubectl create namespace dev
kubectl create namespace qa
```

Next, we create a file called `api-status.yaml` that will be reused for both environments:

```bash
cat <<EOF > api-status.yaml
```

With the following content:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-status
  labels:
    app: api-status
spec:
  replicas: 1
  selector:
    matchLabels:
      app: api-status
  template:
    metadata:
      labels:
        app: api-status
    spec:
      containers:
        - name: podinfo
          image: stefanprodan/podinfo:6.4.0
          ports:
            - containerPort: 9898
          env:
            - name: PODINFO_UI_MESSAGE
              valueFrom:
                fieldRef:
                  fieldPath: metadata.namespace
EOF
```

The `PODINFO_UI_MESSAGE` environment variable is injected using the downward API, which allows a container to read its own Pod metadata at runtime. The `podinfo` application uses this variable to set the `message` field in its JSON response, making it easy to confirm which namespace the Pod is running in.

Notice that the manifest does not include a `namespace` field in the metadata. We will supply the target namespace at apply time using the `-n` flag, which lets us reuse the same manifest for both environments.

To verify the file was created correctly, run:

```bash
cat api-status.yaml
```

Apply the manifest to both namespaces:

```bash
kubectl apply -f api-status.yaml -n dev
kubectl apply -f api-status.yaml -n qa
```

Next, we expose each Deployment as a ClusterIP Service inside its respective namespace:

```bash
kubectl expose deployment api-status \
    -n dev \
    --name=api-status-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=9898
```

```bash
kubectl expose deployment api-status \
    -n qa \
    --name=api-status-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=9898
```

#### Verify resource creation

To verify that the Pods are running in each namespace, execute the following commands:

```bash
kubectl get pods -n dev -l app=api-status
kubectl get pods -n qa -l app=api-status
```

The output for each should look similar to this:

```bash
NAME                          READY   STATUS    RESTARTS   AGE
api-status-7d6c8b4f59-r3n8x   1/1     Running   0          1m
```

To verify that the Services are configured correctly in each namespace, run:

```bash
kubectl get svc -n dev api-status-svc
kubectl get svc -n qa api-status-svc
```

The output for each should look similar to this:

```bash
NAME              TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)   AGE
api-status-svc    ClusterIP   10.96.185.42    <none>        80/TCP    1m
```

Note that the two Services share the same name (`api-status-svc`) but have different Cluster IPs, because they are independent resources in separate namespaces.

#### Test the API status endpoint

To test the dev API status endpoint, create a temporary Pod inside the `dev` namespace and send a request through the Service:

```bash
kubectl run -n dev -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside the busybox Pod, use `wget` to access the API status endpoint through the Service:

```bash
wget -qO- http://api-status-svc
```

The response should be a JSON payload showing pod metadata, with the `message` field set to the namespace the Pod is running in:

```json
{
  "hostname": "api-status-7d6c8b4f59-r3n8x",
  "version": "6.4.0",
  "revision": "",
  "color": "#34577c",
  "logo": "https://raw.githubusercontent.com/stefanprodan/podinfo/gh-pages/cuddle_clap.gif",
  "message": "dev",
  "goos": "linux",
  "goarch": "amd64",
  "runtime": "go1.21.0",
  "num_goroutine": "8",
  "num_cpu": "2"
}
```

To confirm that the response contains the correct namespace, run:

```bash
wget -qO- http://api-status-svc | grep '"message"'
```

The output should show the `dev` namespace:

```json
"message": "dev",
```

Repeat the same test for the `qa` namespace by running the busybox Pod with `-n qa`. The message field should show `qa` instead of `dev`, confirming that each Deployment is running in its own isolated namespace.

#### Verify namespace isolation

To confirm that the short Service name does not resolve across namespaces, create a temporary Pod in the default namespace:

```bash
kubectl run -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside this Pod, attempt to reach the dev API status endpoint using its short service name:

```bash
wget -qO- --timeout=5 http://api-status-svc
```

This fails because short Service names only resolve within the same namespace. Services in other namespaces are reachable using their fully qualified DNS name (`<service>.<namespace>.svc.cluster.local`):

```bash
wget -qO- http://api-status-svc.dev.svc.cluster.local
```

This request succeeds, demonstrating that Kubernetes namespaces scope resource visibility and RBAC, but do not enforce network-level isolation on their own. To restrict cross-namespace traffic, NetworkPolicies must be used in addition to namespaces.

The same can be done to access the QA API status endpoint:

```bash
wget -qO- http://api-status-svc.qa.svc.cluster.local
```

## Task 3: Design and deploy an internal request debugging tool in team-isolated namespaces

Your organization has two independent development teams, `team-alpha` and `team-beta`, that share the same cluster. Each team needs its own instance of an internal request debugging tool so they can inspect HTTP headers and connection details without interfering with each other.

The debugging tool must run as a [traefik/whoami](https://hub.docker.com/r/traefik/whoami) container, which returns a plain-text summary of each incoming request including the server name, hostname, IP address, and headers. The server name will be set to the namespace through the downward API, making it easy to confirm namespace isolation from the response. It does not need to be highly resilient, since brief periods of unavailability are acceptable.

Other services within each namespace need a stable address to reach the debugging tool, but it must not be accessible from outside the cluster.

### Architectural design

The task requires running the same application in two isolated environments, brief downtime is acceptable, and the application must be reachable only from inside each namespace. These constraints drive four design decisions:

1. Two separate Namespaces (`team-alpha` and `team-beta`) provide the isolation boundary. Every Kubernetes resource is scoped to a Namespace, so Deployments, Pods, and Services created in one Namespace are invisible to the other. This lets both environments share the same resource names without conflict.

2. Because the application is a single container and brief downtime is acceptable, a Deployment with one replica per Namespace is enough. Each Deployment creates its own ReplicaSet, which recreates the Pod automatically if it crashes, at the cost of a short period of unavailability that the task explicitly allows.

3. Other services within each Namespace need a stable address to reach the debugging tool. Pod IPs change every time a Pod is recreated, so we place a ClusterIP Service (`request-debug-svc`) in front of the Pod in each Namespace. The Service provides a fixed cluster-internal DNS name and forwards traffic to the Pod. It accepts requests on port `80` and forwards them to the container's port `80`.

4. The application must not be accessible from outside the cluster. A ClusterIP Service has no external port and no route from outside the cluster network, so it satisfies this requirement by design. No Gateway, Ingress, or NodePort is needed.

![Architecture diagram](diagrams_images/namespace-isolated-deployment_task3.png)

The diagram shows the resulting architecture: the `team-alpha` and `team-beta` Namespaces each contain an independent Deployment and ClusterIP Service with the same names. External clients have no path into either environment, while internal services reach the debugging tool through the ClusterIP Service in their own Namespace. Cross-namespace access is possible only via the fully qualified DNS name (`request-debug-svc.<namespace>.svc.cluster.local`), since short Service names resolve only within the same Namespace.

### Implementation

We start by creating the two namespaces:

```bash
kubectl create namespace team-alpha
kubectl create namespace team-beta
```

Next, we create a file called `request-debug.yaml` that will be reused for both environments:

```bash
cat <<EOF > request-debug.yaml
```

With the following content:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: request-debug
  labels:
    app: request-debug
spec:
  replicas: 1
  selector:
    matchLabels:
      app: request-debug
  template:
    metadata:
      labels:
        app: request-debug
    spec:
      containers:
        - name: whoami
          image: traefik/whoami:v1.10
          ports:
            - containerPort: 80
          env:
            - name: WHOAMI_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.namespace
EOF
```

The `WHOAMI_NAME` environment variable is injected using the downward API, which allows a container to read its own Pod metadata at runtime. The `whoami` application uses this variable to override the `Name` field in its plain-text response, making it easy to confirm which namespace the Pod is running in.

Notice that the manifest does not include a `namespace` field in the metadata. We will supply the target namespace at apply time using the `-n` flag, which lets us reuse the same manifest for both environments.

To verify the file was created correctly, run:

```bash
cat request-debug.yaml
```

Apply the manifest to both namespaces:

```bash
kubectl apply -f request-debug.yaml -n team-alpha
kubectl apply -f request-debug.yaml -n team-beta
```

Next, we expose each Deployment as a ClusterIP Service inside its respective namespace:

```bash
kubectl expose deployment request-debug \
    -n team-alpha \
    --name=request-debug-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=80
```

```bash
kubectl expose deployment request-debug \
    -n team-beta \
    --name=request-debug-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=80
```

#### Verify resource creation

To verify that the Pods are running in each namespace, execute the following commands:

```bash
kubectl get pods -n team-alpha -l app=request-debug
kubectl get pods -n team-beta -l app=request-debug
```

The output for each should look similar to this:

```bash
NAME                             READY   STATUS    RESTARTS   AGE
request-debug-5c8f9a7b64-k2v4p   1/1     Running   0          1m
```

To verify that the Services are configured correctly in each namespace, run:

```bash
kubectl get svc -n team-alpha request-debug-svc
kubectl get svc -n team-beta request-debug-svc
```

The output for each should look similar to this:

```bash
NAME                TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)   AGE
request-debug-svc   ClusterIP   10.96.203.17    <none>        80/TCP    1m
```

Note that the two Services share the same name (`request-debug-svc`) but have different Cluster IPs, because they are independent resources in separate namespaces.

#### Test the request debugging tool

To test the team-alpha debugging tool, create a temporary Pod inside the `team-alpha` namespace and send a request through the Service:

```bash
kubectl run -n team-alpha -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside the busybox Pod, use `wget` to access the debugging tool through the Service:

```bash
wget -qO- http://request-debug-svc
```

The response should be a plain-text summary showing request and server details, with the `Name` field set to the namespace the Pod is running in:

```text
Name: team-alpha
Hostname: request-debug-5c8f9a7b64-k2v4p
IP: 127.0.0.1
IP: 10.244.0.12
RemoteAddr: 10.244.0.15:48762
GET / HTTP/1.1
Host: request-debug-svc
User-Agent: Wget
```

To confirm that the response contains the correct namespace, run:

```bash
wget -qO- http://request-debug-svc | grep 'Name:'
```

The output should show the `team-alpha` namespace:

```text
Name: team-alpha
```

Repeat the same test for the `team-beta` namespace by running the busybox Pod with `-n team-beta`. The Name field should show `team-beta` instead of `team-alpha`, confirming that each Deployment is running in its own isolated namespace.

#### Verify namespace isolation

To confirm that the short Service name does not resolve across namespaces, create a temporary Pod in the default namespace:

```bash
kubectl run -it --rm --restart=Never busybox --image=busybox -- sh
```

Inside this Pod, attempt to reach the team-alpha debugging tool using its short service name:

```bash
wget -qO- --timeout=5 http://request-debug-svc
```

This fails because short Service names only resolve within the same namespace. Services in other namespaces are reachable using their fully qualified DNS name (`<service>.<namespace>.svc.cluster.local`):

```bash
wget -qO- http://request-debug-svc.team-alpha.svc.cluster.local
```

This request succeeds, demonstrating that Kubernetes namespaces scope resource visibility and RBAC, but do not enforce network-level isolation on their own. To restrict cross-namespace traffic, NetworkPolicies must be used in addition to namespaces.

The same can be done to access the team-beta debugging tool:

```bash
wget -qO- http://request-debug-svc.team-beta.svc.cluster.local
```
