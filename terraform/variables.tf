variable "hostname" {
  type = string
  description = "The hostname of where fims will be accessible at"
}

variable "image" {
  type = string
  description = "The image to use in the deployment"
}

variable "migration_runner_image" {
  type = string
  description = "Image to use to run database migrations"
}