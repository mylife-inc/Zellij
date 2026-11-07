Reusable content
================

|Product| is the |edition|.

The |product| name can be reused with |docs|_ and |logo|.

The release is |release| (|version|).

Read |section| and download |notes|.

Inline code keeps ``|Product|`` literal; an escaped \|Product| stays literal too.

.. code-block:: rst

   |undefined-in-code|
   .. |ignored| replace:: not a definition
   .. include:: /not/executed

- A list uses |Product| and |docs|_.

.. note::

   A note uses |edition|.

.. list-table:: |Product| features
   :header-rows: 1

   * - Product
     - Icon
   * - |Product|
     - |logo|

.. |Product| replace:: **Zellij**
.. |edition| replace:: |Product| documentation engine
.. |docs| replace:: *Read* the guide
.. _docs: details.html#install
.. |logo| image:: media/logo.svg
   :alt: Substitution logo
   :target: docs_
.. |section| replace:: :ref:`Substituted section <install>`
.. |notes| replace:: :download:`Substituted notes <notes.txt>`
