resource "kubernetes_job_v1" "migrations" {
  metadata {
    namespace = kubernetes_namespace_v1.this.metadata[0].name
    generate_name = "fims-db-migration-"
  }
  spec {
    template {
      metadata {
        
      }
      spec {
        container {
          name = "sqlx"
          image = var.migration_runner_image
          restart_policy = "Never"
          env {
            name = "DATABASE_URL"
            value_from {
              secret_key_ref {
                name = "aws-credentials"
                key = "DATABASE_URL"
              }
            }
          }
        }
      }
    }
    ttl_seconds_after_finished = 60
    backoff_limit = 4
  }
}