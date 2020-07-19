'use strict';

const e = React.createElement;

class AlertExpanded extends React.Component {
    constructor(props) {
        super(props);
        this.alertData = props.alertData;
        this.state = { expanded: true, fired_timestamps: [], awaiting: this.alertData.awaiting };
        this.toggleAwaiting = this.toggleAwaiting.bind(this);
        this.deleteAlert = this.deleteAlert.bind(this);
    }

    componentDidMount() {
        fetch(location.href + '/alerts/v1/read/' + this.alertData.id, {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie,
              }
            })
            .then(response => response.json())
            .then(data => { 
                let new_awaiting = this.alertData.awaiting;
                if (data.alert.fired_timestamps.length != 0) {
                    new_awaiting = data.alert.awaiting;
                }
                this.setState({ expanded: this.expanded, fired_timestamps: data.alert.fired_timestamps, awaiting: new_awaiting });
            });
    }
    
    deleteAlert() {
        fetch(location.href + 'alerts/v1/delete/' + this.alertData.id, {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie
              }
            })
            .then(response => response.json())
            .then(data => { 
                this.setState({ expanded: this.state.expanded, fired_timestamps: this.state.fired_timestamps, awaiting: this.state.awaiting });
            });
    }

    toggleAwaiting() {
        let url = location.href + 'alerts/v1/set_awaiting/' + this.alertData.id;
        if (this.state.awaiting == true) {
            url = location.href + 'alerts/v1/unset_awaiting/' + this.alertData.id;
        }

        fetch(url, {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie
              }
            })
            .then(response => response.json())
            .then(data => { 
                this.setState({ expanded: this.state.expanded, fired_timestamps: this.state.fired_timestamps, awaiting: data.awaiting });
            });
    }

    render() {
        if (this.state.expanded == false) {
            return (
                <Alert alertData = { this.alertData } />
              );
        }

        let className = 'alert-not-awaiting';
        if (this.state.awaiting == true) {
            className = 'alert-awaiting';
        }

        return (
            <div className={className} >
                <div className="row mb-1" key={"e" + this.alertData.id} >
                    <div className="col-2 themed-grid-col" onClick={() => this.setState({ expanded: false, fired_timestamps: this.state.fired_timestamps, awaiting: this.state.awaiting }) }>{this.alertData.id}</div>
                    <div className="col-6 themed-grid-col" onClick={() => this.setState({ expanded: false, fired_timestamps: this.state.fired_timestamps, awaiting: this.state.awaiting }) }>
                    { this.state.fired_timestamps.map(timestamp =>
                        <div>{timestamp}</div>
                    )}
                    </div>
                    <div className="col-4 themed-grid-col">
                        <button onClick={this.toggleAwaiting}>Toggle Awaiting</button>
                        <button onClick={this.deleteAlert}>Delete</button>
                    </div>
                </div>
            </div>
          );
    }
}

class Alert extends React.Component {
    constructor(props) {
        super(props);
        this.alertData = props.alertData;
        this.state = { expanded: false, awaiting: this.alertData.awaiting };
        this.toggleAwaiting = this.toggleAwaiting.bind(this);
        this.deleteAlert = this.deleteAlert.bind(this);
    }

    toggleAwaiting() {
        let url = location.href + 'alerts/v1/set_awaiting/' + this.alertData.id;
        if (this.state.awaiting == true) {
            url = location.href + 'alerts/v1/unset_awaiting/' + this.alertData.id;
        }

        fetch(url, {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie
              }
            })
            .then(response => response.json())
            .then(data => { 
                this.setState({ expanded: this.state.expanded, fired_timestamps: this.state.fired_timestamps, awaiting: data.awaiting });
            });
    }

    deleteAlert() {
        fetch(location.href + 'alerts/v1/delete/' + this.alertData.id, {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie
              }
            })
            .then(response => response.json())
            .then(data => { 
                this.setState({ expanded: this.state.expanded, fired_timestamps: this.state.fired_timestamps, awaiting: this.state.awaiting });
            });
    }

    render() {
        if (this.state.expanded) {
            return (
                <AlertExpanded alertData = { this.alertData } />
              );
        }

        let className = 'alert-not-awaiting';
        if (this.state.awaiting == true) {
            className = 'alert-awaiting';
        }

        return (
            
            <div className={className} >
                <div className="row mb-1" key={this.alertData.id} >
                    <div className="col-2 themed-grid-col" onClick={() => this.setState({ expanded: true, awaiting: this.state.awaiting }) }>{this.alertData.id}</div>
                    <div className="col-6 themed-grid-col" onClick={() => this.setState({ expanded: true, awaiting: this.state.awaiting }) }>{this.alertData.timestamp}</div>
                    <div className="col-4 themed-grid-col">
                            <button onClick={this.toggleAwaiting}>Toggle Awaiting</button>
                            <button onClick={this.deleteAlert}>Delete</button>
                    </div>
                </div>
            </div>
          );
    }
}


class AlertContainer extends React.Component {
    constructor(props) {
      super(props);
      this.state = { alertArray: [] };
    }

    componentDidMount() {
        fetch(location.href + 'alerts/v1/read', {
            method: 'GET',
            headers: {
                'Authorization': 'Bearer ' + document.cookie
              }
        })
        .then(response => response.json())
        .then(data => { 
            this.setState({ alertArray: data.alerts });
        });
    }
  
    render() {
        if (document.cookie === "") {
            return (
                <div>Please log in by setting the auth-key cookie</div>
            );
        } else {
            const { alertArray } = this.state;
  
            return (
              <div key="alblock">
              { alertArray.map(alert =>
                  <Alert alertData = { alert } />
              )}
              </div>
            );
        }

    
    }
}
  
let domContainer = document.querySelector('#alert_container');
ReactDOM.render(<AlertContainer />, domContainer);



class RegisterInput extends React.Component {
    constructor(props) {
        super(props);
        this.state = { value: '' };
    }

    handleChange = (e) => {
        this.setState({value: e.target.value});
    }

    handleKeyDown = (e) => {
        if (e.key === 'Enter') {
            fetch(location.href + 'alerts/v1/set_awaiting/' + this.state.value, {
                method: 'GET',
                headers: {
                    'Authorization': 'Bearer ' + document.cookie
                }
            })
            .then(response => response.json());
            this.setState({value: ''});
        }
    }

    render() {
        return <input value={this.state.value} onChange={this.handleChange} onKeyDown={this.handleKeyDown}  type='text' />
    }
}


let domContainer2 = document.querySelector('#register_input');
ReactDOM.render(<RegisterInput />, domContainer2);
