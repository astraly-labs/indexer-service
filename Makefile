localstack:
	awslocal s3api create-bucket --bucket indexer-service --region eu-west-3 --create-bucket-configuration LocationConstraint=eu-west-3
	awslocal s3api put-object --bucket indexer-service --key apibara-scripts/
	awslocal s3api list-buckets

gcs-init:
	curl -X POST \
		--data-binary '{"name":"test-bucket"}' \
		-H "Content-Type: application/json" \
		"http://localhost:4443/storage/v1/b"

format:
	cargo fmt
	cargo clippy --all -- -D warnings
	cargo clippy --tests --no-deps -- -D warnings