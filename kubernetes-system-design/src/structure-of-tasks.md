# Structure of tasks

Every task in this book follows the same three-part structure: a scenario that sets the context, an architectural design that justifies the solution, and an implementation that walks through the commands.

## Scenario

Each task opens with a short scenario describing what the team needs. The scenario establishes the functional requirement (what the application does), the container image to use, the resilience expectations (whether brief downtime is acceptable or not), and the accessibility constraints (internal-only, externally reachable, etc.). These constraints are what drive the architectural decisions that follow.

## Architectural design

The architectural design section translates the scenario constraints into concrete design decisions. Each decision is linked to a specific constraint and to the Kubernetes resource that satisfies it. For example, if the task allows brief downtime, this section explains why a single-replica Deployment is sufficient. If the application must be reachable only from inside the cluster, it explains why a ClusterIP Service is the right choice and why no Ingress or Gateway is needed.

This section also includes an architecture diagram that shows the resulting resource topology: how external and internal clients interact (or do not interact) with the application, and how traffic flows from the Service into the Pod managed by the Deployment.

## Implementation

The implementation section provides the step-by-step commands to deploy the solution. It is organized into three parts:

1. **Resource creation**: The main `kubectl` commands to create the Kubernetes resources required to implement the architectural design. Each command is explained: why a particular flag or value was chosen, and how it connects back to the architectural design. Where useful, a `--dry-run=client -o yaml` variant of the command is included so the reader can inspect the generated YAML before applying it.

2. **Verify resource creation:**: A list of commands to confirm that the resources were created correctly. This typically includes checking things like whether a Pod is running or a Service has the expected type, ports, and no unintended external IP.

3. **Test the application**: A practical test that validates end-to-end connectivity. This usually involves creating a temporary Pod (such as busybox) inside the cluster and using `wget` to send a request to the Service. The expected response is shown so you can confirm that the application is working as intended.
