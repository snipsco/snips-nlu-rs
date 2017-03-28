# Resources

## Graphs

### `graph_logistic_regression.pb`
This model is a (binary) Logistic Regression `y = sigmoid(W * x)`, where
```
     .2
W =  .3
     .5
```
The input node is called `inputs`, the output node is called `logits`.

### `graph_multiclass_logistic_regression.pb`
This model is a multi-class (5 classes) Logistic Regression `y = softmax(W * x)`, where
```
      .2   .3   .5   .7  .11
W =  .13  .17  .19  .23  .29
     .31  .37  .41  .43  .47
```
The input node is called `inputs`, the output node is called `logits`.

### `graph_crf.pb`
This model is a Conditional Random Field (CRF). The unary potentials are a linear mapping of the inputs with parameter `W`, and transition matrix is `psi`, where
```
      .2   .3   .5   .7  .11
W =  .13  .17  .19  .23  .29
     .31  .37  .41  .43  .47

        0.   1.   2.   3.   4.
        5.   6.   7.   8.   9.
psi =  10.  11.  12.  13.  14.
       15.  16.  17.  18.  19.
       20.  21.  22.  23.  24.
```
The input node is called `inputs`, the output node is called `logits` and the transition matrix node is called `psi`.

## Resources

To generate a protobuf file representing a TensorFlow graph, you can run the following python script:
```python
import tensorflow as tf
import numpy as np

# Your graph definition here
x = tf.placeholder(tf.float32, shape=(2, 3), name='inputs')
w_np = np.array([ .2,  .3,  .5,  .7, .11,
                 .13, .17, .19, .23, .29,
                 .31, .37, .41, .43, .47], dtype='float32').reshape((3, 5))
w = tf.constant(w_np, dtype=tf.float32)
y = tf.matmul(x, w, name='logits')

with tf.Session() as sess:
    tf.train.write_graph(sess.graph_def, '.', 'graph.pb', as_text=False)
```
