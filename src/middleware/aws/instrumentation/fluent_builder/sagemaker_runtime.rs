/// AWS SageMaker Runtime fluent builder instrumentation implementations
use super::{utils::*, *};

// InvokeEndpoint
impl<'a> AwsBuilderInstrument<'a> for InvokeEndpointFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let endpoint_name = self.get_endpoint_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_target_model()
                .as_attribute("aws.sagemaker.target_model"),
            self.get_target_variant()
                .as_attribute("aws.sagemaker.target_variant"),
            self.get_inference_component_name()
                .as_attribute("aws.sagemaker.inference_component_name"),
        ];
        SageMakerRuntimeSpanBuilder::invoke_endpoint(endpoint_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for InvokeEndpointOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.invoked_production_variant()
                .as_attribute("aws.sagemaker.invoked_production_variant"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_sagemakerruntime::operation::invoke_endpoint);

// InvokeEndpointAsync
impl<'a> AwsBuilderInstrument<'a> for InvokeEndpointAsyncFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let endpoint_name = self.get_endpoint_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_inference_id()
                .as_attribute("aws.sagemaker.inference_id"),
            self.get_input_location()
                .as_attribute("aws.sagemaker.input_location"),
        ];
        SageMakerRuntimeSpanBuilder::invoke_endpoint_async(endpoint_name)
            .attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for InvokeEndpointAsyncOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.inference_id()
                .as_attribute("aws.sagemaker.inference_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_sagemakerruntime::operation::invoke_endpoint_async);

// InvokeEndpointWithResponseStream
impl<'a> AwsBuilderInstrument<'a> for InvokeEndpointWithResponseStreamFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let endpoint_name = self.get_endpoint_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_target_variant()
                .as_attribute("aws.sagemaker.target_variant"),
            self.get_inference_component_name()
                .as_attribute("aws.sagemaker.inference_component_name"),
        ];
        SageMakerRuntimeSpanBuilder::invoke_endpoint_with_response_stream(endpoint_name)
            .attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for InvokeEndpointWithResponseStreamOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.invoked_production_variant()
                .as_attribute("aws.sagemaker.invoked_production_variant"),
        ]
    }
}
instrument_aws_operation!(
    aws_sdk_sagemakerruntime::operation::invoke_endpoint_with_response_stream
);
