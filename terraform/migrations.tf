resource "kubernetes_job_v1" "migrations" {
  metadata {
    name = "fims-db-migration"
    namespace = kubernetes_namespace_v1.this.metadata[0].name
  }
  spec {    
    template {
      metadata {
        generate_name = "fims-db-migration-"
      }
      spec {
        container {
          name = "sqlx"
          image = var.migration_runner_image
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
    backoff_limit = 4
  }
}