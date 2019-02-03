import * as React from 'react';
interface SendFormProps { onSend: Fn<string> }

type Fn<T, R=void> = (x:T) => R;

export class SendForm extends React.Component<SendFormProps, { content: string }> {

    private input?: HTMLInputElement ;

    constructor(props: SendFormProps) {
        super(props);
    }

    handleContent = ({ target: { value: content } }: React.ChangeEvent<HTMLInputElement>) => {
      this.setState({ content })
    };

    send = () => {
        this.props.onSend(this.state.content);
        this.input!.value = "";
    }

    render() {
        return (
          <React.Fragment>
            <input
              type="text"
              onChange={this.handleContent}
              onKeyPress={e => e.key === "Enter" && this.send()}
              ref={e => (this.input = e ? e : undefined)}
            />
            <button onClick={this.send}>send</button>
          </React.Fragment>
        );
    }
}