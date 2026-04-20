# Structure of the book

The book is organized as set of tasks, with a specific structure, that get grouped into different topics, depending on the nature of the problem, and the kind of solutions that are presented. 

## The topics

The book is currently structured around 5 different topics. For each topic, a list of tasks is provided, with the structure outlined in the following subsection.

The topics are: 
1. Single-container application deployment 
1. Multi-container application deployment 
1. Namespace-isolated application deployment
1. Resilient application deployment
1. Internet-facing application deployment

### Single-container application deployment

In this topic, we cover the basics of deploying a simple application running in one container, including defining a Pod, the Deployment with a single container image and the ClusterIP service to ensure a virtual IP to the application, for internal access.

### Multi-container application deployment

Here, we focus on applications that require multiple containers in the same Pod, such as an app container plus a helper sidecar, sharing networking and storage. We will focus here on the knowledge of multi-container pod patterns and container lifecycle, as well as understanding of shared volumes between containers. 

### Namespace-isolated application deployment

In this topic, we explore how to organize and isolate workloads using Kubernetes namespaces, applying deployments, services, and policies within dedicated logical environments. In particular, we will show how to design and deploy the same application with its internal Service into separate Namespaces to simulate staging and production environments. We include specific competences on Namespace isolation, resource scoping, and deploying objects into specific Namespaces.

### Resilient application deployment

We show, in this topic, tasks that are meant to design robust applications by configuring health checks, update strategies, replicas, and self-healing mechanisms so the system can recover from failures automatically. In particulare, we will show how to design and deploy an application and configure it to run with multiple replicas across the cluster by using ReplicaSets.

### Internet-facing application deployment

In this last topic, we show how to expose applications to external users through Kubernetes Gateway API with path-based routing rules, illustrating it with a Gateway sitting in front of Services.

## The tasks

Every task in this book follows the same three-part structure:
1. A scenario that sets the context.
1. An architectural design that justifies the solution.
1. An implementation that walks through the commands.

### Scenario

Each task opens with a short scenario describing what the team needs. The scenario establishes the functional requirement (what the application does), the container image to use, the resilience expectations (whether brief downtime is acceptable or not), and the accessibility constraints (internal-only, externally reachable, etc.). These constraints are what drive the architectural decisions that follow.

### Architectural design

The architectural design section translates the scenario constraints into concrete design decisions. Each decision is linked to a specific constraint and to the Kubernetes resource that satisfies it. For example, if the task allows brief downtime, this section explains why a single-replica Deployment is sufficient. If the application must be reachable only from inside the cluster, it explains why a ClusterIP Service is the right choice and why no Ingress or Gateway is needed.

This section also includes an architecture diagram that shows the resulting resource topology: how external and internal clients interact (or do not interact) with the application, and how traffic flows from the Service into the Pod managed by the Deployment.

### Implementation

The implementation section provides the step-by-step commands to deploy the solution. It is organized into three parts:

1. **Resource creation**: The main `kubectl` commands to create the Kubernetes resources required to implement the architectural design. Each command is explained: why a particular flag or value was chosen, and how it connects back to the architectural design. Where useful, a `--dry-run=client -o yaml` variant of the command is included so the reader can inspect the generated YAML before applying it.

2. **Verify resource creation:**: A list of commands to confirm that the resources were created correctly. This typically includes checking things like whether a Pod is running or a Service has the expected type, ports, and no unintended external IP.

3. **Test the application**: A practical test that validates end-to-end connectivity. This usually involves creating a temporary Pod (such as busybox) inside the cluster and using `wget` to send a request to the Service. The expected response is shown so you can confirm that the application is working as intended.
