# Single-container deployment

Design and deploy a simple single-container application with a service for internal access.

This category includes the following learning objectives:
- Understanding of Pods.
- sUnderstanding of Deployments.
- Understanding of ClusterIP services.

## Task 1: Design and deploy an internal dashboard

Your team needs an internal monitoring dashboard running inside the cluster to know at any time the node and namespace they are working on.

The dashboard must be packaged as a single container image ([paulbouwer/hello-kubernetes dashboard](https://hub.docker.com/r/paulbouwer/hello-kubernetes)) and does not need to be highly resilient, as it is fine if the dashboard is not available for a short period of time.

However, other services inside the cluster will need a stable address to reach it, so Pod IPs alone will not be enough. Make sure the dashboard is strictly for internal use, it should not be accessible from outside the cluster.

### Architectural design

### Implementation

We start by creating a Deployment with a single replica (default) as the task description says a short period of unavailability is acceptable, so one instance is enough. We point it at the `paulbouwer/hello-kubernetes:1.10` image and declare that the container listens on port `8080`. `create deployment` automatically creates a label `app=hello-dashboard` on the Pods, which will be useful later when we create the Service.

```bash
kubectl create deployment hello-dashboard \
    --image=paulbouwer/hello-kubernetes:1.10 \
    --port=8080
```

Next, we expose the Deployment as a ClusterIP Service. ClusterIP is the right choice here because it gives other services inside the cluster a stable address to reach the dashboard, while keeping it completely inaccessible from outside.

We use `kubectl expose` rather than creating the Service manually using `kubectl create service clusterip` as it automatically sets the selector to match the Deployment's Pods, which is exactly the wiring we need. The Service listens on port `80` and forwards traffic to the container's port `8080`.

```bash
kubectl expose deployment hello-dashboard \
    --name=hello-dashboard-svc \
    --type=ClusterIP \
    --port=80 \
    --target-port=8080
```

#### Verify resource creation

To verify that the Pod is running, we can run the following command that filters Pods by the `app=hello-dashboard` label, which is automatically set by `kubectl create deployment`:

```bash
kubectl get pods -l app=hello-dashboard
```

The output should be similar to this:

```bash
NAME                               READY   STATUS    RESTARTS   AGE
hello-dashboard-6bfbf8b67c-jv8tv   1/1     Running   0          16m
```

To check that the Service is correctly set up, we can run:

```bash
kubectl get svc hello-dashboard-svc
```

The output should look similar to this:

```bash
NAME                  TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)   AGE
hello-dashboard-svc   ClusterIP   10.111.28.77   <none>        80/TCP    15
```

From which, we can confirm that internal access to the dashboard is available at `http://hello-dashboard-svc:80` and no external access is possible as there is no external IP assigned.

#### Test the dashboard

To test the dashboard, we can create a temporary Pod using [busybox](https://hub.docker.com/_/busybox):

```bash
kubectl run -it --rm --restart=Never busybox --image=busybox sh
```

And, inside the busybox Pod, we can use `wget` to access the dashboard through the Service's ClusterIP. The dashboard should respond with an HTML page containing information about the cluster.

```bash
wget -qO- http://hello-dashboard-svc
```

The dashboard's HTML content should look like the example below:

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
                    <tr><th>namespace:</th><td>-</td></tr>
                    <tr><th>pod:</th><td>hello-dashboard-6bfbf8b67c-jv8tv</td></tr>
                    <tr><th>node:</th><td>- (Linux 6.8.0-94-generic)</td></tr>
                </table>
                </div>
            </div>
        </div>
    </body>
</html>
```
